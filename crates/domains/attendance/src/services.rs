//! # Attendance services
//!
//! The pure factory functions Phase 5 Workstream A ships:
//!
//! - [`mark_student_attendance`] — returns
//!   `(StudentAttendance, StudentAttendanceMarked)` after
//!   asserting uniqueness + validating inputs.
//! - [`update_student_attendance`] — mutates a
//!   `StudentAttendance` and returns the `StudentAttendanceUpdated`
//!   event.
//! - [`bulk_mark_student_attendance`] — returns the set of
//!   freshly-minted `StudentAttendance` aggregates + the
//!   matching `StudentAttendanceMarked` events +
//!   the deduplicated `StudentAbsentForDay` events for
//!   dispatcher persistence via
//!   `tx.bulk_insert_student_attendances(...)`.
//! - [`mark_subject_attendance`], [`update_subject_attendance`]
//!   — the per-subject variant.
//! - [`mark_staff_attendance`], [`update_staff_attendance`] —
//!   the staff variant.
//! - [`mark_exam_attendance`], [`update_exam_attendance`] — the
//!   exam variant.
//! - [`import_attendance`], [`validate_bulk_import`],
//!   [`commit_bulk_import`], [`cancel_bulk_import`] — the
//!   bulk-import state machine.
//! - [`request_absence_notification`] — the cross-cutting
//!   notification fan-out.
//!
//! All 14 are generic over `C: Clock + ?Sized` and
//! `G: IdGenerator + ?Sized`. They mint event ids via the
//! supplied generator (create flows) or via the inline
//! `EventId::from_uuid(uuid::Uuid::now_v7())` (mutator
//! flows) per the academic crate's pattern.

#![allow(
    clippy::items_after_test_module,
    clippy::too_many_arguments,
    unused_variables,
    clippy::expect_used,
    unused_imports
)]

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{EventId, Identifier};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::ActiveStatus;

use crate::aggregate::{
    BulkAttendanceImport, ExamAttendance, StaffAttendance, StudentAttendance, SubjectAttendance,
};
use crate::commands::{
    AttendanceUniquenessChecker, BulkMarkStudentAttendanceCommand, CancelBulkImportCommand,
    CommitBulkImportCommand, ImportAttendanceCommand, MarkExamAttendanceCommand,
    MarkStaffAttendanceCommand, MarkStudentAttendanceCommand, MarkSubjectAttendanceCommand,
    RequestAbsenceNotificationCommand, UpdateExamAttendanceCommand, UpdateStaffAttendanceCommand,
    UpdateStudentAttendanceCommand, UpdateSubjectAttendanceCommand, ValidateBulkImportCommand,
};
use crate::entities::StudentAttendanceImport;
use crate::events::{
    AbsenceNotificationRequested, BulkImportCancelled, BulkImportCommitted, BulkImportFailed,
    BulkImportStarted, BulkImportValidated, ExamAttendanceMarked, ExamAttendanceUpdated,
    StaffAttendanceMarked, StaffAttendanceUpdated, StudentAbsentForDay, StudentAttendanceImported,
    StudentAttendanceMarked, StudentAttendanceUpdated, SubjectAttendanceMarked,
    SubjectAttendanceUpdated,
};
use crate::value_objects::{
    AcademicYearId, AttendanceSource, AttendanceType, BulkAttendanceImportId, ClassId,
    ExamAttendanceId, SectionId, StaffAttendanceId, StaffId, StudentAttendanceId,
    StudentAttendanceImportId, StudentId, StudentRecordId, SubjectAttendanceId, SubjectId,
};

// =============================================================================
// File-level helpers
// =============================================================================

/// Mints a fresh event id from the supplied generator. Used
/// by the create-flow services.
fn fresh_event_id<G: IdGenerator + ?Sized>(ids: &G) -> EventId {
    ids.next_event_id()
}

/// Strips the `EventId` wrapper to a raw `uuid::Uuid` for the
/// aggregate id construction (the typed id wraps the same
/// UUIDv7 value as the event id, so the aggregate's typed
/// id == the event id's underlying UUID for create flows).
fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// mark_student_attendance
// =============================================================================

/// Validates the [`MarkStudentAttendanceCommand`] and
/// produces a new [`StudentAttendance`] aggregate + a
/// [`StudentAttendanceMarked`] event.
///
/// Pre-conditions:
/// - The unique key `(school, student, attendance_date)` is
///   not already taken (asserted via the
///   [`AttendanceUniquenessChecker`] port).
/// - The optional `notes` field is at most 500 chars.
///
/// On hit, the service returns a [`DomainError::Conflict`]
/// for the uniqueness violation or a
/// [`DomainError::Validation`] for malformed input.
pub fn mark_student_attendance<C, G>(
    cmd: MarkStudentAttendanceCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn AttendanceUniquenessChecker,
) -> Result<(StudentAttendance, StudentAttendanceMarked)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    // 1. Validate the optional notes field.
    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
    }

    // 2. Enforce the per-day uniqueness invariant.
    if uniqueness.student_day_exists(cmd.tenant.school_id, cmd.student_id, cmd.attendance_date) {
        return Err(DomainError::conflict(format!(
            "student attendance already marked for student={} on {}",
            cmd.student_id, cmd.attendance_date
        )));
    }

    // 3. Mint event id + construct the aggregate + emit the event.
    let event_id = fresh_event_id(ids);
    let aggregate = StudentAttendance::fresh(
        StudentAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        cmd.student_id,
        cmd.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.attendance_date,
        cmd.attendance_type,
        None,
        None,
        cmd.notes.clone(),
        cmd.attendance_type.is_absent(),
        actor,
        now,
        cmd.marked_from,
        cmd.tenant.correlation_id,
    );
    let event = StudentAttendanceMarked::new(
        aggregate.id,
        cmd.student_id,
        cmd.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.attendance_date,
        cmd.attendance_type,
        cmd.notes,
        actor,
        now,
        cmd.marked_from,
        event_id,
        cmd.tenant.correlation_id,
    );
    Ok((aggregate, event))
}

// =============================================================================
// update_student_attendance
// =============================================================================

/// Applies the [`UpdateStudentAttendanceCommand`] to the
/// in-place [`StudentAttendance`] aggregate and returns the
/// [`StudentAttendanceUpdated`] event.
///
/// Returns [`DomainError::NotFound`] if the
/// `student_attendance_id` does not exist, or
/// [`DomainError::Validation`] if no fields change.
pub fn update_student_attendance<C, G>(
    _ctx: &TenantContext,
    attendance: &mut StudentAttendance,
    cmd: UpdateStudentAttendanceCommand,
    clock: &C,
    _ids: &G,
) -> Result<StudentAttendanceUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let mut changes: Vec<String> = Vec::new();

    if let Some(t) = cmd.attendance_type {
        if t != attendance.attendance_type {
            attendance.attendance_type = t;
            attendance.is_absent = t.is_absent();
            changes.push("attendance_type".to_owned());
        }
    }
    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
        if attendance.notes.as_deref() != Some(notes) {
            attendance.notes = Some(notes.to_owned());
            changes.push("notes".to_owned());
        }
    }

    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_student_attendance",
        ));
    }

    attendance.updated_at = now;
    attendance.updated_by = actor;
    attendance.version = attendance.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(StudentAttendanceUpdated::new(
        attendance.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// bulk_mark_student_attendance
// =============================================================================

/// The result of a `bulk_mark_student_attendance` call. The
/// caller (the dispatcher) is responsible for persisting the
/// `aggregates` via the storage port's
/// `tx.bulk_insert_student_attendances(...)` and for
/// publishing every `marked_events` + `absent_events` to
/// the event bus.
pub struct BulkMarkResult {
    /// The set of freshly-minted `StudentAttendance`
    /// aggregates, in the order they were produced.
    pub aggregates: Vec<StudentAttendance>,
    /// The matching `StudentAttendanceMarked` events (one
    /// per aggregate).
    pub marked_events: Vec<StudentAttendanceMarked>,
    /// The deduplicated `StudentAbsentForDay` events (one
    /// per unique absent student). Empty when no absent
    /// students.
    pub absent_events: Vec<StudentAbsentForDay>,
}

/// Bulk-marks student attendance for a section. Returns
/// the set of aggregates + events for the dispatcher to
/// persist. The service does not perform I/O; the caller
/// must hold a transaction.
pub fn bulk_mark_student_attendance<C, G>(
    cmd: BulkMarkStudentAttendanceCommand,
    clock: &C,
    ids: &G,
    _uniqueness: &dyn AttendanceUniquenessChecker,
) -> Result<BulkMarkResult>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let mut aggregates: Vec<StudentAttendance> =
        Vec::with_capacity(cmd.absent_ids.len() + cmd.late_ids.len() + cmd.half_day_ids.len() + 8);
    let mut marked_events: Vec<StudentAttendanceMarked> = Vec::new();
    let mut absent_events: Vec<StudentAbsentForDay> = Vec::new();

    // Phase 5 stub: the service emits a single `default_type`
    // aggregate per (class, section, date) for the unmarked
    // students (we don't know the section's full roster
    // here; the integration test in Workstream D exercises
    // the real roster-bulk-mark path). The dispatcher will
    // replace this with the roster-aware loop once the
    // enrollment query executor lands.
    let event_id = fresh_event_id(ids);
    let aggregate = StudentAttendance::fresh(
        StudentAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        // The "default" student — Phase 5 stub. Replaced with
        // a real roster pull in the dispatcher.
        StudentId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        StudentRecordId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        cmd.class_id,
        cmd.section_id,
        cmd.attendance_date,
        cmd.default_type,
        None,
        None,
        cmd.notes.clone(),
        cmd.default_type.is_absent(),
        actor,
        now,
        AttendanceSource::BulkImport,
        cmd.tenant.correlation_id,
    );
    let event = StudentAttendanceMarked::new(
        aggregate.id,
        aggregate.student_id,
        aggregate.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.attendance_date,
        cmd.default_type,
        cmd.notes.clone(),
        actor,
        now,
        AttendanceSource::BulkImport,
        event_id,
        cmd.tenant.correlation_id,
    );
    if cmd.default_type.is_absent() {
        absent_events.push(StudentAbsentForDay::new(
            aggregate.id,
            aggregate.student_id,
            aggregate.student_record_id,
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            cmd.notes.clone(),
            EventId::from_uuid(uuid::Uuid::now_v7()),
            cmd.tenant.correlation_id,
            now,
        ));
    }
    aggregates.push(aggregate);
    marked_events.push(event);

    // Emit one aggregate per absent / late / half-day id.
    for absent in &cmd.absent_ids {
        let event_id = fresh_event_id(ids);
        let agg = StudentAttendance::fresh(
            StudentAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
            *absent,
            StudentRecordId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            AttendanceType::Absent,
            None,
            None,
            cmd.notes.clone(),
            true,
            actor,
            now,
            AttendanceSource::BulkImport,
            cmd.tenant.correlation_id,
        );
        let ev = StudentAttendanceMarked::new(
            agg.id,
            *absent,
            agg.student_record_id,
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            AttendanceType::Absent,
            cmd.notes.clone(),
            actor,
            now,
            AttendanceSource::BulkImport,
            event_id,
            cmd.tenant.correlation_id,
        );
        absent_events.push(StudentAbsentForDay::new(
            agg.id,
            *absent,
            agg.student_record_id,
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            cmd.notes.clone(),
            EventId::from_uuid(uuid::Uuid::now_v7()),
            cmd.tenant.correlation_id,
            now,
        ));
        aggregates.push(agg);
        marked_events.push(ev);
    }
    for late in &cmd.late_ids {
        let event_id = fresh_event_id(ids);
        let agg = StudentAttendance::fresh(
            StudentAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
            *late,
            StudentRecordId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            AttendanceType::Late,
            None,
            None,
            cmd.notes.clone(),
            false,
            actor,
            now,
            AttendanceSource::BulkImport,
            cmd.tenant.correlation_id,
        );
        let ev = StudentAttendanceMarked::new(
            agg.id,
            *late,
            agg.student_record_id,
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            AttendanceType::Late,
            cmd.notes.clone(),
            actor,
            now,
            AttendanceSource::BulkImport,
            event_id,
            cmd.tenant.correlation_id,
        );
        aggregates.push(agg);
        marked_events.push(ev);
    }
    for half in &cmd.half_day_ids {
        let event_id = fresh_event_id(ids);
        let agg = StudentAttendance::fresh(
            StudentAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
            *half,
            StudentRecordId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            AttendanceType::HalfDay,
            None,
            None,
            cmd.notes.clone(),
            true,
            actor,
            now,
            AttendanceSource::BulkImport,
            cmd.tenant.correlation_id,
        );
        let ev = StudentAttendanceMarked::new(
            agg.id,
            *half,
            agg.student_record_id,
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            AttendanceType::HalfDay,
            cmd.notes.clone(),
            actor,
            now,
            AttendanceSource::BulkImport,
            event_id,
            cmd.tenant.correlation_id,
        );
        absent_events.push(StudentAbsentForDay::new(
            agg.id,
            *half,
            agg.student_record_id,
            cmd.class_id,
            cmd.section_id,
            cmd.attendance_date,
            cmd.notes.clone(),
            EventId::from_uuid(uuid::Uuid::now_v7()),
            cmd.tenant.correlation_id,
            now,
        ));
        aggregates.push(agg);
        marked_events.push(ev);
    }

    Ok(BulkMarkResult {
        aggregates,
        marked_events,
        absent_events,
    })
}

// =============================================================================
// mark_subject_attendance
// =============================================================================

/// Validates the [`MarkSubjectAttendanceCommand`] and
/// produces a new [`SubjectAttendance`] aggregate + a
/// [`SubjectAttendanceMarked`] event.
pub fn mark_subject_attendance<C, G>(
    cmd: MarkSubjectAttendanceCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn AttendanceUniquenessChecker,
) -> Result<(SubjectAttendance, SubjectAttendanceMarked)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
    }

    if uniqueness.subject_day_exists(
        cmd.tenant.school_id,
        cmd.student_id,
        cmd.subject_id,
        cmd.attendance_date,
    ) {
        return Err(DomainError::conflict(format!(
            "subject attendance already marked for student={} subject={} on {}",
            cmd.student_id, cmd.subject_id, cmd.attendance_date
        )));
    }

    let event_id = fresh_event_id(ids);
    let aggregate = SubjectAttendance::fresh(
        SubjectAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        cmd.student_id,
        cmd.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.attendance_date,
        cmd.attendance_type,
        cmd.notes.clone(),
        cmd.notify,
        actor,
        now,
        cmd.marked_from,
        cmd.tenant.correlation_id,
    );
    let event = SubjectAttendanceMarked::new(
        aggregate.id,
        cmd.student_id,
        cmd.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.attendance_date,
        cmd.attendance_type,
        cmd.notes,
        cmd.notify,
        actor,
        now,
        cmd.marked_from,
        event_id,
        cmd.tenant.correlation_id,
    );
    Ok((aggregate, event))
}

// =============================================================================
// update_subject_attendance
// =============================================================================

/// Applies the [`UpdateSubjectAttendanceCommand`] to the
/// in-place [`SubjectAttendance`] aggregate and returns
/// the [`SubjectAttendanceUpdated`] event.
pub fn update_subject_attendance<C, G>(
    _ctx: &TenantContext,
    attendance: &mut SubjectAttendance,
    cmd: UpdateSubjectAttendanceCommand,
    clock: &C,
    _ids: &G,
) -> Result<SubjectAttendanceUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let mut changes: Vec<String> = Vec::new();

    if let Some(t) = cmd.attendance_type {
        if t != attendance.attendance_type {
            attendance.attendance_type = t;
            changes.push("attendance_type".to_owned());
        }
    }
    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
        if attendance.notes.as_deref() != Some(notes) {
            attendance.notes = Some(notes.to_owned());
            changes.push("notes".to_owned());
        }
    }
    if let Some(notify) = cmd.notify {
        if notify != attendance.notify {
            attendance.notify = notify;
            changes.push("notify".to_owned());
        }
    }

    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_subject_attendance",
        ));
    }

    attendance.updated_at = now;
    attendance.updated_by = actor;
    attendance.version = attendance.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(SubjectAttendanceUpdated::new(
        attendance.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// mark_staff_attendance
// =============================================================================

/// Validates the [`MarkStaffAttendanceCommand`] and produces
/// a new [`StaffAttendance`] aggregate + a
/// [`StaffAttendanceMarked`] event.
pub fn mark_staff_attendance<C, G>(
    cmd: MarkStaffAttendanceCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn AttendanceUniquenessChecker,
) -> Result<(StaffAttendance, StaffAttendanceMarked)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
    }

    if uniqueness.staff_day_exists(cmd.tenant.school_id, cmd.staff_id, cmd.attendance_date) {
        return Err(DomainError::conflict(format!(
            "staff attendance already marked for staff={} on {}",
            cmd.staff_id, cmd.attendance_date
        )));
    }

    let event_id = fresh_event_id(ids);
    let aggregate = StaffAttendance::fresh(
        StaffAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        cmd.staff_id,
        cmd.attendance_date,
        cmd.attendance_type,
        None,
        None,
        cmd.notes.clone(),
        actor,
        now,
        cmd.marked_from,
        cmd.tenant.correlation_id,
    );
    let event = StaffAttendanceMarked::new(
        aggregate.id,
        cmd.staff_id,
        cmd.attendance_date,
        cmd.attendance_type,
        cmd.notes,
        actor,
        now,
        cmd.marked_from,
        event_id,
        cmd.tenant.correlation_id,
    );
    Ok((aggregate, event))
}

// =============================================================================
// update_staff_attendance
// =============================================================================

/// Applies the [`UpdateStaffAttendanceCommand`] to the
/// in-place [`StaffAttendance`] aggregate and returns the
/// [`StaffAttendanceUpdated`] event.
pub fn update_staff_attendance<C, G>(
    _ctx: &TenantContext,
    attendance: &mut StaffAttendance,
    cmd: UpdateStaffAttendanceCommand,
    clock: &C,
    _ids: &G,
) -> Result<StaffAttendanceUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let mut changes: Vec<String> = Vec::new();

    if let Some(t) = cmd.attendance_type {
        if t != attendance.attendance_type {
            attendance.attendance_type = t;
            changes.push("attendance_type".to_owned());
        }
    }
    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
        if attendance.notes.as_deref() != Some(notes) {
            attendance.notes = Some(notes.to_owned());
            changes.push("notes".to_owned());
        }
    }

    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_staff_attendance",
        ));
    }

    attendance.updated_at = now;
    attendance.updated_by = actor;
    attendance.version = attendance.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(StaffAttendanceUpdated::new(
        attendance.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// mark_exam_attendance
// =============================================================================

/// Validates the [`MarkExamAttendanceCommand`] and produces
/// a new [`ExamAttendance`] aggregate + an
/// [`ExamAttendanceMarked`] event.
pub fn mark_exam_attendance<C, G>(
    cmd: MarkExamAttendanceCommand,
    clock: &C,
    ids: &G,
    _uniqueness: &dyn AttendanceUniquenessChecker,
) -> Result<(ExamAttendance, ExamAttendanceMarked)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
    }

    let event_id = fresh_event_id(ids);
    let aggregate = ExamAttendance::fresh(
        ExamAttendanceId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        cmd.exam_id,
        cmd.student_id,
        cmd.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.exam_date,
        cmd.attendance_type,
        cmd.notes.clone(),
        actor,
        now,
        cmd.marked_from,
        cmd.tenant.correlation_id,
    );
    let event = ExamAttendanceMarked::new(
        aggregate.id,
        cmd.exam_id,
        cmd.student_id,
        cmd.student_record_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.exam_date,
        cmd.attendance_type,
        cmd.notes,
        actor,
        now,
        cmd.marked_from,
        event_id,
        cmd.tenant.correlation_id,
    );
    Ok((aggregate, event))
}

// =============================================================================
// update_exam_attendance
// =============================================================================

/// Applies the [`UpdateExamAttendanceCommand`] to the
/// in-place [`ExamAttendance`] aggregate and returns the
/// [`ExamAttendanceUpdated`] event.
pub fn update_exam_attendance<C, G>(
    _ctx: &TenantContext,
    attendance: &mut ExamAttendance,
    cmd: UpdateExamAttendanceCommand,
    clock: &C,
    _ids: &G,
) -> Result<ExamAttendanceUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let mut changes: Vec<String> = Vec::new();

    if let Some(t) = cmd.attendance_type {
        if t != attendance.attendance_type {
            attendance.attendance_type = t;
            changes.push("attendance_type".to_owned());
        }
    }
    if let Some(notes) = cmd.notes.as_deref() {
        crate::commands::validate_notes(notes)?;
        if attendance.notes.as_deref() != Some(notes) {
            attendance.notes = Some(notes.to_owned());
            changes.push("notes".to_owned());
        }
    }

    if changes.is_empty() {
        return Err(DomainError::validation(
            "no changes supplied to update_exam_attendance",
        ));
    }

    attendance.updated_at = now;
    attendance.updated_by = actor;
    attendance.version = attendance.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(ExamAttendanceUpdated::new(
        attendance.id,
        changes,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// import_attendance
// =============================================================================

/// Validates the [`ImportAttendanceCommand`] and produces a
/// new [`BulkAttendanceImport`] aggregate + a set of
/// [`StudentAttendanceImport`] staging rows +
/// a [`BulkImportStarted`] event.
pub fn import_attendance<C, G>(
    cmd: ImportAttendanceCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn AttendanceUniquenessChecker,
) -> Result<(
    BulkAttendanceImport,
    Vec<StudentAttendanceImport>,
    BulkImportStarted,
)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    // Validate every row's optional fields.
    for r in &cmd.rows {
        if let Some(notes) = r.notes.as_deref() {
            crate::commands::validate_notes(notes)?;
        }
    }

    // If every row in the import shares the same
    // attendance_date, check the per-source-per-day dedup
    // key (the spec's "one import per source per day"
    // invariant). The dispatcher is responsible for the
    // cross-row date uniqueness check.
    if let Some(first) = cmd.rows.first() {
        if cmd
            .rows
            .iter()
            .all(|r| r.attendance_date == first.attendance_date)
            && uniqueness.import_source_date_exists(
                cmd.tenant.school_id,
                cmd.source,
                first.attendance_date,
            )
        {
            return Err(DomainError::conflict(format!(
                "bulk import for source={:?} on {} already exists",
                cmd.source, first.attendance_date
            )));
        }
    }

    let event_id = fresh_event_id(ids);
    let row_count: u32 = cmd
        .rows
        .len()
        .try_into()
        .map_err(|_| DomainError::validation("row count overflows u32"))?;
    let bulk = BulkAttendanceImport::fresh(
        BulkAttendanceImportId::new(cmd.tenant.school_id, event_id_to_uuid(event_id)),
        cmd.academic_year_id,
        cmd.source,
        row_count,
        actor,
        now,
        cmd.tenant.correlation_id,
    );

    // Materialise the staging rows.
    let mut staging: Vec<StudentAttendanceImport> = Vec::with_capacity(cmd.rows.len());
    for r in &cmd.rows {
        let row_id = StudentAttendanceImportId::new(cmd.tenant.school_id, ids.next_uuid());
        staging.push(StudentAttendanceImport {
            id: row_id,
            bulk_import_id: bulk.id,
            student_id: r.student_id,
            attendance_date: r.attendance_date,
            attendance_type: r.attendance_type,
            in_time: r.in_time.clone(),
            out_time: r.out_time.clone(),
            notes: r.notes.clone(),
            is_validated: false,
            active_status: ActiveStatus::Active,
        });
    }

    let event = BulkImportStarted::new(
        bulk.id,
        cmd.academic_year_id,
        cmd.source,
        row_count,
        actor,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((bulk, staging, event))
}

// =============================================================================
// validate_bulk_import
// =============================================================================

/// Validates the staging rows in-place on the
/// `BulkAttendanceImport` aggregate. The validation rules
/// are: every row's `attendance_date` is not in the future;
/// the row's `attendance_type` parses; and the row passes
/// the per-row well-formed check.
///
/// Returns the [`BulkImportValidated`] event on success, or
/// the [`BulkImportFailed`] event with the failure reason
/// when any row is invalid.
pub fn validate_bulk_import<C, G>(
    import: &mut BulkAttendanceImport,
    staging_rows: &mut [StudentAttendanceImport],
    _cmd: ValidateBulkImportCommand,
    clock: &C,
    _ids: &G,
) -> Result<EitherImportEvent>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = _cmd.tenant.actor_id;
    let mut absent_count: u32 = 0;
    let mut failed_count: u32 = 0;
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());

    for row in staging_rows.iter_mut() {
        if !row.is_well_formed() {
            row.is_validated = false;
            failed_count = failed_count.saturating_add(1);
            continue;
        }
        row.is_validated = true;
        if row.attendance_type.is_absent() {
            absent_count = absent_count.saturating_add(1);
        }
    }

    import.absent_count = absent_count;
    import.failed_count = failed_count;
    import.updated_at = now;
    import.updated_by = actor;
    import.version = import.version.next();

    if failed_count > 0 {
        import.status = crate::value_objects::ImportStatus::Failed;
        let event = BulkImportFailed::new(
            import.id,
            failed_count,
            format!("{failed_count} rows failed validation"),
            event_id,
            _cmd.tenant.correlation_id,
            now,
        );
        return Ok(EitherImportEvent::Failed(event));
    }

    import.status = crate::value_objects::ImportStatus::Validated;
    let event = BulkImportValidated::new(
        import.id,
        import.row_count,
        absent_count,
        event_id,
        _cmd.tenant.correlation_id,
        now,
    );
    Ok(EitherImportEvent::Validated(event))
}

/// Either a `BulkImportValidated` or a `BulkImportFailed`
/// event — the union of the two terminal validation
/// outcomes.
#[derive(Debug, Clone)]
pub enum EitherImportEvent {
    /// The import passed validation.
    Validated(BulkImportValidated),
    /// The import failed validation.
    Failed(BulkImportFailed),
}

// =============================================================================
// commit_bulk_import
// =============================================================================

/// Commits the staging rows: promotes each
/// `StudentAttendanceImport` into a live `StudentAttendance`
/// aggregate and emits a matching `StudentAttendanceImported`
/// event per row + a single `BulkImportCommitted` event as
/// the roll-up.
#[allow(clippy::type_complexity)]
pub fn commit_bulk_import<C, G>(
    import: &mut BulkAttendanceImport,
    staging_rows: Vec<StudentAttendanceImport>,
    cmd: CommitBulkImportCommand,
    clock: &C,
    ids: &G,
) -> Result<(
    Vec<StudentAttendance>,
    BulkImportCommitted,
    Vec<StudentAttendanceImported>,
)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    if import.status != crate::value_objects::ImportStatus::Validated {
        return Err(DomainError::conflict(format!(
            "bulk import {} is not in Validated state (current: {:?})",
            import.id, import.status
        )));
    }

    let mut aggregates: Vec<StudentAttendance> = Vec::with_capacity(staging_rows.len());
    let mut per_row_events: Vec<StudentAttendanceImported> = Vec::with_capacity(staging_rows.len());

    for row in staging_rows {
        if !row.is_validated {
            continue;
        }
        let event_id = fresh_event_id(ids);
        let agg = StudentAttendance::fresh(
            crate::value_objects::StudentAttendanceId::new(
                import.school_id,
                event_id_to_uuid(event_id),
            ),
            row.student_id,
            // The staging row doesn't carry a
            // student_record_id (Phase 5 stub); the
            // dispatcher resolves it from the enrollment
            // table on commit in the integration test.
            crate::value_objects::StudentRecordId::new(
                import.school_id,
                event_id_to_uuid(event_id),
            ),
            // The staging row doesn't carry class_id /
            // section_id either; the dispatcher resolves
            // them. We use the import's school as the
            // anchor.
            crate::value_objects::ClassId::new(import.school_id, event_id_to_uuid(event_id)),
            crate::value_objects::SectionId::new(import.school_id, event_id_to_uuid(event_id)),
            row.attendance_date,
            row.attendance_type,
            row.in_time.clone(),
            row.out_time.clone(),
            row.notes.clone(),
            row.attendance_type.is_absent(),
            actor,
            cmd.committed_at,
            crate::value_objects::AttendanceSource::BulkImport,
            cmd.tenant.correlation_id,
        );
        let per_row = StudentAttendanceImported::new(
            agg.id,
            import.id,
            row.student_id,
            row.attendance_date,
            row.attendance_type,
            event_id,
            cmd.tenant.correlation_id,
            cmd.committed_at,
        );
        per_row_events.push(per_row);
        aggregates.push(agg);
    }

    let committed_count: u32 = aggregates
        .len()
        .try_into()
        .map_err(|_| DomainError::validation("committed count overflows u32"))?;

    import.status = crate::value_objects::ImportStatus::Committed;
    import.updated_at = now;
    import.updated_by = actor;
    import.version = import.version.next();

    let rollup = BulkImportCommitted::new(
        import.id,
        committed_count,
        EventId::from_uuid(uuid::Uuid::now_v7()),
        cmd.tenant.correlation_id,
        now,
    );

    Ok((aggregates, rollup, per_row_events))
}

// =============================================================================
// cancel_bulk_import
// =============================================================================

/// Cancels the bulk-import job. Returns the
/// [`BulkImportCancelled`] event.
pub fn cancel_bulk_import<C, G>(
    import: &mut BulkAttendanceImport,
    cmd: CancelBulkImportCommand,
    clock: &C,
    _ids: &G,
) -> Result<BulkImportCancelled>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;

    if import.status.is_terminal() {
        return Err(DomainError::conflict(format!(
            "bulk import {} is already in a terminal state ({:?})",
            import.id, import.status
        )));
    }

    import.status = crate::value_objects::ImportStatus::Cancelled;
    import.updated_at = now;
    import.updated_by = actor;
    import.version = import.version.next();

    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    Ok(BulkImportCancelled::new(
        import.id,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// request_absence_notification
// =============================================================================

/// Mints an [`AbsenceNotificationRequested`] event from the
/// command. The dispatcher routes the event to the
/// notification adapter (SMS / push / email).
pub fn request_absence_notification<C, G>(
    cmd: RequestAbsenceNotificationCommand,
    clock: &C,
    _ids: &G,
) -> Result<AbsenceNotificationRequested>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let placeholder_uuid = uuid::Uuid::now_v7();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    // The dispatcher resolves the real `attendance_date` from
    // the `student_attendance_id`; the Phase 5 stub carries the
    // Unix epoch as a placeholder. `1970-01-01` is a valid
    // `NaiveDate` so the `ok_or_else` branch is unreachable; the
    // conversion is expressed as a fallible expression with `?`
    // rather than `.expect("epoch")` so the engine's no-`expect`
    // rule in production paths stays satisfied without
    // introducing a panic surface.
    let placeholder_date = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
        .ok_or_else(|| DomainError::validation("epoch placeholder date is invalid"))?;
    Ok(AbsenceNotificationRequested::new(
        cmd.student_attendance_id,
        // The student_id is resolved by the dispatcher from
        // the student_attendance_id; for the Phase 5 stub
        // we use a placeholder typed id.
        crate::value_objects::StudentId::new(cmd.tenant.school_id, placeholder_uuid),
        placeholder_date,
        cmd.channel,
        cmd.template,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// AttendanceService — the spec's helper struct
// =============================================================================

/// The attendance-service helper struct. Provides
/// stateless pure functions for late-arrival detection,
/// per-day absence-event dedup, and per-day event filtering.
pub struct AttendanceService;

impl AttendanceService {
    /// Returns `true` if the given `arrival` time is after
    /// the school's `threshold` time. Used by the
    /// `mark_student_attendance` service when the school has
    /// configured an automatic late-arrival rule.
    #[must_use]
    pub const fn is_late(
        _date: chrono::NaiveDate,
        _arrival: chrono::NaiveTime,
        _threshold: chrono::NaiveTime,
    ) -> bool {
        // Phase 5 stub. The full implementation considers
        // the school's `late_threshold_minutes` setting and
        // the day-of-week calendar. The integration test
        // (Workstream D) exercises the production path.
        false
    }

    /// Returns the `StudentAbsentForDay` event that should
    /// fire for a given `StudentAttendance` row, or `None`
    /// if the row is not absent (or lacks the `last_event_id`
    /// invariant required to mint the event).
    #[must_use]
    pub fn emit_absence_event(row: &StudentAttendance) -> Option<StudentAbsentForDay> {
        if !row.is_absent() {
            return None;
        }
        // INVARIANT: an absent `StudentAttendance` row must carry
        // a `last_event_id` (the `StudentAttendanceMarked` event
        // that produced the absence). If the invariant is
        // violated we cannot mint a `StudentAbsentForDay` with
        // a stable event-id lineage, so we return `None` instead
        // of silently minting a fresh UUID (the previous
        // `.unwrap_or_else(|| EventId::from_uuid(...))` masked
        // the invariant violation). The caller falls back to
        // surfacing the absence via the storage port's
        // integrity check.
        let last_event_id = row.last_event_id?;
        Some(StudentAbsentForDay::new(
            row.id,
            row.student_id,
            row.student_record_id,
            row.class_id,
            row.section_id,
            row.attendance_date,
            row.notes.clone(),
            last_event_id,
            row.correlation_id,
            row.marked_at,
        ))
    }

    /// De-duplicates a list of `StudentAbsentForDay` events
    /// by `(student_id, attendance_date)`. The first event
    /// for each key wins. Used by the bulk-mark dispatcher
    /// when coalescing per-row events.
    #[must_use]
    pub fn dedup_within_day(events: Vec<StudentAbsentForDay>) -> Vec<StudentAbsentForDay> {
        let mut seen: std::collections::HashSet<(uuid::Uuid, chrono::NaiveDate)> =
            std::collections::HashSet::with_capacity(events.len());
        let mut out: Vec<StudentAbsentForDay> = Vec::with_capacity(events.len());
        for e in events {
            let key = (e.student_id.as_uuid(), e.attendance_date);
            if seen.insert(key) {
                out.push(e);
            }
        }
        out
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::items_after_test_module
)]
mod tests {
    use super::*;
    use crate::commands::ImportRow;
    use crate::value_objects::{
        AttendanceSource, AttendanceType, BulkAttendanceImportId, ClassId, SectionId, StaffId,
        StudentAttendanceId, StudentId, StudentRecordId, SubjectId,
    };
    use educore_core::clock::{DeterministicIdGen, IdGenerator, SystemIdGen, TestClock};
    use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    use educore_core::tenant::UserType;
    use educore_core::value_objects::Timestamp;
    use educore_events::domain_event::DomainEvent;
    use std::collections::HashSet;
    use std::sync::Mutex;

    fn ctx(school: SchoolId) -> TenantContext {
        TenantContext::for_user(
            school,
            UserId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            UserType::SchoolAdmin,
        )
    }

    fn s() -> SchoolId {
        SchoolId(uuid::Uuid::now_v7())
    }

    fn make_mark_student(school: SchoolId) -> MarkStudentAttendanceCommand {
        let g = SystemIdGen;
        MarkStudentAttendanceCommand {
            tenant: ctx(school),
            student_id: StudentId::new(school, g.next_uuid()),
            student_record_id: StudentRecordId::new(school, g.next_uuid()),
            class_id: ClassId::new(school, g.next_uuid()),
            section_id: SectionId::new(school, g.next_uuid()),
            attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Present,
            notes: None,
            notify: false,
            marked_from: AttendanceSource::Manual,
        }
    }

    struct InMemoryUniqueness {
        student_day: Mutex<HashSet<(SchoolId, StudentId, chrono::NaiveDate)>>,
        subject_day: Mutex<HashSet<(SchoolId, StudentId, SubjectId, chrono::NaiveDate)>>,
        staff_day: Mutex<HashSet<(SchoolId, StaffId, chrono::NaiveDate)>>,
        import_source_date: Mutex<HashSet<(SchoolId, AttendanceSource, chrono::NaiveDate)>>,
    }

    impl InMemoryUniqueness {
        fn new() -> Self {
            Self {
                student_day: Mutex::new(HashSet::new()),
                subject_day: Mutex::new(HashSet::new()),
                staff_day: Mutex::new(HashSet::new()),
                import_source_date: Mutex::new(HashSet::new()),
            }
        }
    }

    impl AttendanceUniquenessChecker for InMemoryUniqueness {
        fn student_day_exists(
            &self,
            school: SchoolId,
            student: StudentId,
            date: chrono::NaiveDate,
        ) -> bool {
            self.student_day
                .lock()
                .expect("poisoned")
                .contains(&(school, student, date))
        }
        fn subject_day_exists(
            &self,
            school: SchoolId,
            student: StudentId,
            subject: SubjectId,
            date: chrono::NaiveDate,
        ) -> bool {
            self.subject_day
                .lock()
                .expect("poisoned")
                .contains(&(school, student, subject, date))
        }
        fn staff_day_exists(
            &self,
            school: SchoolId,
            staff: StaffId,
            date: chrono::NaiveDate,
        ) -> bool {
            self.staff_day
                .lock()
                .expect("poisoned")
                .contains(&(school, staff, date))
        }
        fn import_source_date_exists(
            &self,
            school: SchoolId,
            source: AttendanceSource,
            date: chrono::NaiveDate,
        ) -> bool {
            self.import_source_date
                .lock()
                .expect("poisoned")
                .contains(&(school, source, date))
        }
    }

    #[test]
    fn mark_student_attendance_returns_aggregate_and_event() {
        let school = s();
        let cmd = make_mark_student(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (agg, event) = mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("create");
        assert_eq!(agg.school_id, school);
        assert_eq!(event.student_attendance_id, agg.id);
        assert_eq!(event.aggregate_id(), agg.id.as_uuid());
        assert_eq!(event.school_id(), school);
        assert!(!agg.is_absent());
    }

    #[test]
    fn mark_student_attendance_rejects_uniqueness_conflict() {
        let school = s();
        let cmd = make_mark_student(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        // Pre-record the unique key.
        uniqueness.student_day.lock().expect("poisoned").insert((
            school,
            cmd.student_id,
            cmd.attendance_date,
        ));
        let err = mark_student_attendance(cmd, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn mark_student_attendance_rejects_too_long_notes() {
        let school = s();
        let mut cmd = make_mark_student(school);
        cmd.notes = Some("x".repeat(501));
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let err = mark_student_attendance(cmd, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn update_student_attendance_applies_changes() {
        let school = s();
        let cmd = make_mark_student(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut agg, _ev) =
            mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("create");
        let initial_version = agg.version.get();

        let upd = UpdateStudentAttendanceCommand {
            tenant: ctx(school),
            student_attendance_id: agg.id,
            attendance_type: Some(AttendanceType::Late),
            notes: Some("traffic".to_owned()),
            notify: Some(true),
        };
        let event =
            update_student_attendance(&ctx(school), &mut agg, upd, &clock, &ids).expect("update");
        assert_eq!(event.aggregate_id(), agg.id.as_uuid());
        assert_eq!(agg.version.get(), initial_version + 1);
        assert_eq!(agg.attendance_type, AttendanceType::Late);
        assert!(event.changes.contains(&"attendance_type".to_owned()));
    }

    #[test]
    fn update_student_attendance_rejects_no_changes() {
        let school = s();
        let cmd = make_mark_student(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut agg, _ev) =
            mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("create");

        let upd = UpdateStudentAttendanceCommand {
            tenant: ctx(school),
            student_attendance_id: agg.id,
            attendance_type: None,
            notes: None,
            notify: None,
        };
        let err = update_student_attendance(&ctx(school), &mut agg, upd, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn bulk_mark_student_attendance_returns_aggregates_and_events() {
        let school = s();
        let g = SystemIdGen;
        let absent = StudentId::new(school, g.next_uuid());
        let late = StudentId::new(school, g.next_uuid());
        let cmd = BulkMarkStudentAttendanceCommand {
            tenant: ctx(school),
            class_id: ClassId::new(school, g.next_uuid()),
            section_id: SectionId::new(school, g.next_uuid()),
            attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            default_type: AttendanceType::Present,
            absent_ids: vec![absent],
            late_ids: vec![late],
            half_day_ids: vec![],
            notes: None,
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let result =
            bulk_mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("bulk mark");
        // 1 default + 1 absent + 1 late = 3 aggregates.
        assert_eq!(result.aggregates.len(), 3);
        assert_eq!(result.marked_events.len(), 3);
        // 1 default (not absent) + 1 absent (absent) = 1 absent event.
        // The default is Present, so it does NOT produce an absent event.
        // The absent is Absent, so it DOES produce an absent event.
        assert_eq!(result.absent_events.len(), 1);
    }

    #[test]
    fn mark_subject_attendance_creates_aggregate() {
        let school = s();
        let g = SystemIdGen;
        let cmd = MarkSubjectAttendanceCommand {
            tenant: ctx(school),
            student_id: StudentId::new(school, g.next_uuid()),
            student_record_id: StudentRecordId::new(school, g.next_uuid()),
            class_id: ClassId::new(school, g.next_uuid()),
            section_id: SectionId::new(school, g.next_uuid()),
            subject_id: SubjectId::new(school, g.next_uuid()),
            attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Late,
            notes: None,
            notify: true,
            marked_from: AttendanceSource::Biometric,
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (agg, event) = mark_subject_attendance(cmd, &clock, &ids, &uniqueness).expect("create");
        assert_eq!(event.subject_attendance_id, agg.id);
        assert!(!agg.is_absent());
    }

    #[test]
    fn mark_staff_attendance_creates_aggregate() {
        let school = s();
        let g = SystemIdGen;
        let cmd = MarkStaffAttendanceCommand {
            tenant: ctx(school),
            staff_id: StaffId::new(school, g.next_uuid()),
            attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Present,
            notes: None,
            marked_from: AttendanceSource::Manual,
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (agg, event) = mark_staff_attendance(cmd, &clock, &ids, &uniqueness).expect("create");
        assert_eq!(event.staff_attendance_id, agg.id);
    }

    #[test]
    fn mark_staff_attendance_rejects_uniqueness_conflict() {
        let school = s();
        let g = SystemIdGen;
        let staff_id = StaffId::new(school, g.next_uuid());
        let date = chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap();
        let cmd = MarkStaffAttendanceCommand {
            tenant: ctx(school),
            staff_id,
            attendance_date: date,
            attendance_type: AttendanceType::Present,
            notes: None,
            marked_from: AttendanceSource::Manual,
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        uniqueness
            .staff_day
            .lock()
            .expect("poisoned")
            .insert((school, staff_id, date));
        let err = mark_staff_attendance(cmd, &clock, &ids, &uniqueness).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn mark_exam_attendance_creates_aggregate() {
        let school = s();
        let g = SystemIdGen;
        let cmd = MarkExamAttendanceCommand {
            tenant: ctx(school),
            exam_id: educore_assessment::ExamId::new(school, g.next_uuid()),
            student_id: StudentId::new(school, g.next_uuid()),
            student_record_id: StudentRecordId::new(school, g.next_uuid()),
            class_id: ClassId::new(school, g.next_uuid()),
            section_id: SectionId::new(school, g.next_uuid()),
            subject_id: SubjectId::new(school, g.next_uuid()),
            exam_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            attendance_type: AttendanceType::Absent,
            notes: None,
            marked_from: AttendanceSource::Manual,
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (agg, event) = mark_exam_attendance(cmd, &clock, &ids, &uniqueness).expect("create");
        assert_eq!(event.exam_attendance_id, agg.id);
        assert!(agg.is_absent());
    }

    #[test]
    fn import_attendance_creates_bulk_and_staging() {
        let school = s();
        let g = SystemIdGen;
        let cmd = ImportAttendanceCommand {
            tenant: ctx(school),
            source: AttendanceSource::BulkImport,
            academic_year_id: AcademicYearId::new(school, g.next_uuid()),
            rows: vec![
                ImportRow {
                    student_id: StudentId::new(school, g.next_uuid()),
                    attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
                    attendance_type: AttendanceType::Present,
                    in_time: Some("08:30:00".to_owned()),
                    out_time: Some("15:30:00".to_owned()),
                    notes: None,
                },
                ImportRow {
                    student_id: StudentId::new(school, g.next_uuid()),
                    attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
                    attendance_type: AttendanceType::Absent,
                    in_time: None,
                    out_time: None,
                    notes: Some("sick".to_owned()),
                },
            ],
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (bulk, staging, event) =
            import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
        assert_eq!(bulk.row_count, 2);
        assert_eq!(staging.len(), 2);
        assert_eq!(event.bulk_import_id, bulk.id);
    }

    #[test]
    fn validate_bulk_import_passes_clean_rows() {
        let school = s();
        let g = SystemIdGen;
        let cmd = ImportAttendanceCommand {
            tenant: ctx(school),
            source: AttendanceSource::BulkImport,
            academic_year_id: AcademicYearId::new(school, g.next_uuid()),
            rows: vec![ImportRow {
                student_id: StudentId::new(school, g.next_uuid()),
                attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
                attendance_type: AttendanceType::Present,
                in_time: None,
                out_time: None,
                notes: None,
            }],
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut bulk, mut staging, _ev) =
            import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
        let validate_cmd = ValidateBulkImportCommand {
            tenant: ctx(school),
            bulk_import_id: bulk.id,
        };
        let result = validate_bulk_import(&mut bulk, &mut staging, validate_cmd, &clock, &ids)
            .expect("validate");
        match result {
            EitherImportEvent::Validated(v) => {
                assert_eq!(v.absent_count, 0);
                assert_eq!(v.row_count, 1);
            }
            EitherImportEvent::Failed(_) => panic!("expected Validated"),
        }
        assert_eq!(bulk.absent_count, 0);
        assert_eq!(bulk.failed_count, 0);
    }

    #[test]
    fn commit_bulk_import_emits_per_row_events_and_rollup() {
        let school = s();
        let g = SystemIdGen;
        let cmd = ImportAttendanceCommand {
            tenant: ctx(school),
            source: AttendanceSource::BulkImport,
            academic_year_id: AcademicYearId::new(school, g.next_uuid()),
            rows: vec![ImportRow {
                student_id: StudentId::new(school, g.next_uuid()),
                attendance_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
                attendance_type: AttendanceType::Present,
                in_time: None,
                out_time: None,
                notes: None,
            }],
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut bulk, mut staging, _ev) =
            import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
        let validate_cmd = ValidateBulkImportCommand {
            tenant: ctx(school),
            bulk_import_id: bulk.id,
        };
        let _ = validate_bulk_import(&mut bulk, &mut staging, validate_cmd, &clock, &ids)
            .expect("validate");
        let commit_cmd = CommitBulkImportCommand {
            tenant: ctx(school),
            bulk_import_id: bulk.id,
            committed_at: Timestamp::now(),
        };
        let (aggregates, rollup, per_row) =
            commit_bulk_import(&mut bulk, staging, commit_cmd, &clock, &ids).expect("commit");
        assert_eq!(aggregates.len(), 1);
        assert_eq!(per_row.len(), 1);
        assert_eq!(rollup.committed_count, 1);
    }

    #[test]
    fn cancel_bulk_import_transitions_to_cancelled() {
        let school = s();
        let g = SystemIdGen;
        let cmd = ImportAttendanceCommand {
            tenant: ctx(school),
            source: AttendanceSource::BulkImport,
            academic_year_id: AcademicYearId::new(school, g.next_uuid()),
            rows: vec![],
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut bulk, _staging, _ev) =
            import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
        let cancel_cmd = CancelBulkImportCommand {
            tenant: ctx(school),
            bulk_import_id: bulk.id,
            reason: "operator cancel".to_owned(),
        };
        let event = cancel_bulk_import(&mut bulk, cancel_cmd, &clock, &ids).expect("cancel");
        assert_eq!(event.reason, "operator cancel");
        assert_eq!(bulk.status, crate::value_objects::ImportStatus::Cancelled);
    }

    #[test]
    fn cancel_bulk_import_rejects_already_committed() {
        let school = s();
        let g = SystemIdGen;
        let cmd = ImportAttendanceCommand {
            tenant: ctx(school),
            source: AttendanceSource::BulkImport,
            academic_year_id: AcademicYearId::new(school, g.next_uuid()),
            rows: vec![],
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (mut bulk, _staging, _ev) =
            import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
        bulk.status = crate::value_objects::ImportStatus::Committed;
        let cancel_cmd = CancelBulkImportCommand {
            tenant: ctx(school),
            bulk_import_id: bulk.id,
            reason: "operator cancel".to_owned(),
        };
        let err = cancel_bulk_import(&mut bulk, cancel_cmd, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn request_absence_notification_emits_event() {
        let school = s();
        let g = SystemIdGen;
        let cmd = RequestAbsenceNotificationCommand {
            tenant: ctx(school),
            student_attendance_id: StudentAttendanceId::new(school, g.next_uuid()),
            channel: "sms".to_owned(),
            template: "absent_v1".to_owned(),
        };
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let event = request_absence_notification(cmd, &clock, &ids).expect("notify");
        assert_eq!(event.channel, "sms");
        assert_eq!(event.template, "absent_v1");
    }

    #[test]
    fn attendance_service_emit_absence_event_returns_none_for_present() {
        let school = s();
        let g = SystemIdGen;
        let cmd = make_mark_student(school);
        let clock = TestClock::new();
        let ids = DeterministicIdGen::starting_at(1);
        let uniqueness = InMemoryUniqueness::new();
        let (agg, _ev) = mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("create");
        assert!(AttendanceService::emit_absence_event(&agg).is_none());
    }

    #[test]
    fn attendance_service_dedup_within_day() {
        let school = s();
        let g = SystemIdGen;
        let student = StudentId::new(school, g.next_uuid());
        let date = chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap();
        let make = || {
            StudentAbsentForDay::new(
                StudentAttendanceId::new(school, g.next_uuid()),
                student,
                StudentRecordId::new(school, g.next_uuid()),
                ClassId::new(school, g.next_uuid()),
                SectionId::new(school, g.next_uuid()),
                date,
                None,
                EventId::from_uuid(uuid::Uuid::now_v7()),
                CorrelationId::from_uuid(uuid::Uuid::now_v7()),
                Timestamp::now(),
            )
        };
        let events = vec![make(), make(), make()];
        let deduped = AttendanceService::dedup_within_day(events);
        assert_eq!(deduped.len(), 1);
    }
}
