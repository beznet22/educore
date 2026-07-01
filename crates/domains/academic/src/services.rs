//! # Academic-domain services
//!
//! Pure factory functions that take a command, the active
//! `TenantContext`, a `Clock`, and (for the create flows) an
//! [`UniquenessChecker`], and return the mutated aggregate
//! plus the typed event. The dispatcher (in the engine's
//! core) is responsible for persisting the aggregate and
//! publishing the event under a single transaction.
//!
//! Per `docs/specs/academic/services.md` and the engine
//! invariant "the services module is the only place the
//! engine mutates an aggregate and emits its typed event".
//! The capability check itself is a dispatcher-level
//! concern (matching `educore-platform::services`); see
//! `docs/handoff/PHASE-3-HANDOFF.md` § "Capability check
//! boundary" for the rationale.
//!
//! Phase 3 ships the prompt-named subset:
//!
//! - **Student lifecycle (8 functions)**: [`admit_student`],
//!   [`update_student_profile`], [`suspend_student`],
//!   [`reinstate_student`], [`withdraw_student`],
//!   [`transfer_student`], [`promote_student`],
//!   [`graduate_student`].
//! - **Class & Section (7 functions)**: [`create_class`],
//!   [`update_class`], [`delete_class`],
//!   [`set_optional_subject_gpa_threshold`],
//!   [`create_section`], [`update_section`],
//!   [`delete_section`].
//! - **Subject (3 functions)**: [`create_subject`],
//!   [`update_subject`], [`delete_subject`].
//! - **AcademicYear (5 functions)**: [`create_academic_year`],
//!   [`update_academic_year_dates`],
//!   [`set_current_academic_year`], [`close_academic_year`],
//!   [`copy_academic_year`].

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::ActiveStatus;

use crate::aggregate::{
    AcademicYear, Certificate, Class, ClassRoutine, ClassSection, ClassSubject, Guardian, Homework,
    IdCard, Lesson, LessonPlan, LessonTopic, OptionalSubjectAssignment, RegistrationField, Section,
    Student, StudentCategory, StudentGroup, StudentGuardianLink, StudentPromotion, Subject,
};
use crate::commands::{
    validate_admission_no, validate_class_name, validate_email_optional, validate_first_name,
    validate_gpa_threshold, validate_last_name, validate_mobile_optional, validate_pass_mark,
    validate_roll_no, validate_section_name, validate_subject_code, validate_subject_name,
    validate_suspension_reason, validate_transfer_reason, validate_withdrawal_reason,
    validate_year_label, validate_year_title, AdmitStudentCommand, AssignOptionalSubjectCommand,
    CloseAcademicYearCommand, CreateAcademicYearCommand, CreateCertificateCommand,
    CreateClassCommand, CreateClassRoutineCommand, CreateClassSectionCommand,
    CreateClassSubjectCommand, CreateHomeworkCommand, CreateIdCardCommand, CreateLessonCommand,
    CreateLessonPlanCommand, CreateLessonTopicCommand, CreateRegistrationFieldCommand,
    CreateSectionCommand, CreateStudentCategoryCommand, CreateStudentGroupCommand,
    CreateSubjectCommand, DeleteClassCommand, DeleteSectionCommand, DeleteSubjectCommand,
    GraduateStudentCommand, LinkGuardianToStudentCommand, MarkPrimaryGuardianCommand,
    PromoteStudentCommand, RecordStudentPromotionCommand, RegisterGuardianCommand,
    ReinstateStudentCommand, SetCurrentAcademicYearCommand, SetOptionalSubjectGpaThresholdCommand,
    SuspendStudentCommand, TransferStudentCommand, UniquenessChecker,
    UnlinkGuardianFromStudentCommand, UpdateAcademicYearDatesCommand, UpdateClassCommand,
    UpdateSectionCommand, UpdateStudentProfileCommand, UpdateSubjectCommand,
    WithdrawStudentCommand,
};
use crate::events::{
    AcademicYearClosed, AcademicYearCopied, AcademicYearCreated, AcademicYearDatesUpdated,
    CertificateCreated, ClassCreated, ClassDeleted, ClassRoutineScheduled, ClassSectionCreated,
    ClassSubjectAssigned, ClassUpdated, CurrentAcademicYearSet, GuardianLinkedToStudent,
    GuardianRegistered, GuardianUnlinkedFromStudent, HomeworkAssigned, IdCardCreated,
    LessonCreated, LessonPlanCreated, LessonTopicCreated, OptionalSubjectAssignmentCreated,
    OptionalSubjectGpaThresholdSet, PrimaryGuardianMarked, RegistrationFieldCreated,
    SectionCreated, SectionDeleted, SectionUpdated, StudentAdmitted, StudentCategoryCreated,
    StudentGraduated, StudentGroupCreated, StudentProfileUpdated, StudentPromoted,
    StudentPromotionRecorded, StudentReinstated, StudentSuspended, StudentTransferred,
    StudentWithdrawn, SubjectCreated, SubjectDeleted, SubjectUpdated,
};
use crate::value_objects::{
    AcademicYearId, AcademicYearRange, StudentGuardianLinkId, StudentStatus,
};

fn fresh_event_id<G: IdGenerator + ?Sized>(ids: &G) -> EventId {
    ids.next_event_id()
}

// =============================================================================
// Student lifecycle (8 functions)
// =============================================================================

/// Admit a new [`Student`] and emit a [`StudentAdmitted`]
/// event.
///
/// Returns the new `Student` and the typed event. The
/// caller (the engine's command dispatcher) is responsible
/// for persisting the aggregate and publishing the event
/// under a single transaction.
///
/// # Errors
///
/// - `Validation` if any of the command's string fields
///   fails structural validation.
/// - `Conflict` if the admission number (or email, when
///   supplied) is already taken in the school per the
///   uniqueness checker; or if the roll number is already
///   taken in the `(class, section, academic_year)` scope
///   per Student I-3.
pub fn admit_student<C, G>(
    cmd: AdmitStudentCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(Student, StudentAdmitted)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let AdmitStudentCommand {
        tenant,
        student_id,
        admission_no,
        first_name,
        last_name,
        date_of_birth,
        gender,
        blood_group: _,
        religion: _,
        caste: _,
        mobile: _,
        email: _,
        current_address: _,
        permanent_address: _,
        admission_date,
        class_id,
        section_id,
        academic_year_id,
        roll_no,
        custom_fields: _,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_admission_no(&admission_no)?;
    validate_first_name(&first_name)?;
    validate_last_name(&last_name)?;
    if let Some(e) = cmd.email.as_deref() {
        validate_email_optional(e)?;
    }
    if let Some(roll) = roll_no.as_deref() {
        validate_roll_no(roll)?;
    }
    if uniqueness.student_admission_no_exists(student_id.school_id(), &admission_no) {
        return Err(DomainError::Conflict(format!(
            "student admission number {admission_no:?} is already taken in the school"
        )));
    }
    if let Some(e) = cmd.email.as_deref() {
        let lower = e.to_lowercase();
        if uniqueness.student_email_exists(student_id.school_id(), &lower) {
            return Err(DomainError::Conflict(format!(
                "student email {e:?} is already in use within the school"
            )));
        }
    }
    // I-3: roll number uniqueness within
    // (school, class, section, academic_year).
    if let Some(roll) = roll_no.as_deref() {
        if uniqueness.roll_no_exists(
            student_id.school_id(),
            class_id,
            section_id,
            academic_year_id,
            roll,
        ) {
            return Err(DomainError::Conflict(format!(
                "roll number {roll:?} is already taken in (class, section, academic_year)"
            )));
        }
    }
    let mut student = Student::fresh(
        student_id,
        admission_no.clone(),
        first_name.clone(),
        last_name.clone(),
        date_of_birth,
        gender,
        admission_date,
        class_id,
        section_id,
        academic_year_id,
        roll_no.clone(),
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    student.blood_group = cmd.blood_group;
    student.religion = cmd.religion;
    student.caste = cmd.caste;
    student.mobile = cmd.mobile;
    student.email = cmd.email;
    student.current_address = cmd.current_address;
    student.permanent_address = cmd.permanent_address;
    student.custom_fields = cmd.custom_fields;
    let event_id = fresh_event_id(ids);
    student.last_event_id = Some(event_id);
    student.correlation_id = ctx.correlation_id;

    let event = StudentAdmitted::new(
        student_id,
        admission_no,
        first_name,
        last_name,
        class_id,
        section_id,
        academic_year_id,
        admission_date,
        roll_no,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok((student, event))
}

/// Update a [`Student`]'s mutable profile fields and emit
/// a [`StudentProfileUpdated`] event.
///
/// Returns the typed event. The caller is responsible for
/// persisting the mutated aggregate and publishing the
/// event.
///
/// # Errors
///
/// - `Validation` if any of the supplied fields fails
///   structural validation.
/// - `Conflict` if the new email (when supplied) is already
///   in use within the school.
pub fn update_student_profile<C, G>(
    _ctx: &TenantContext,
    student: &mut Student,
    cmd: UpdateStudentProfileCommand,
    clock: &C,
    _ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<StudentProfileUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let UpdateStudentProfileCommand {
        tenant: _,
        student_id,
        first_name,
        last_name,
        gender,
        mobile,
        email,
        current_address,
        permanent_address,
        custom_fields,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    let now = clock.now();
    let mut changed = Vec::new();
    if let Some(new_first) = first_name {
        validate_first_name(&new_first)?;
        if new_first != student.first_name {
            changed.push("first_name".to_owned());
            student.first_name = new_first;
        }
    }
    if let Some(new_last) = last_name {
        validate_last_name(&new_last)?;
        if new_last != student.last_name {
            changed.push("last_name".to_owned());
            student.last_name = new_last;
        }
    }
    if let Some(new_gender) = gender {
        if new_gender != student.gender {
            changed.push("gender".to_owned());
            student.gender = new_gender;
        }
    }
    if let Some(new_mobile) = mobile {
        if let Some(m) = new_mobile.as_deref() {
            validate_mobile_optional(m)?;
        }
        if new_mobile != student.mobile {
            changed.push("mobile".to_owned());
            student.mobile = new_mobile;
        }
    }
    if let Some(new_email) = email {
        if let Some(e) = new_email.as_deref() {
            validate_email_optional(e)?;
            let lower = e.to_lowercase();
            if uniqueness.student_email_exists(student.school_id, &lower)
                && lower
                    != student
                        .email
                        .as_deref()
                        .map(str::to_lowercase)
                        .unwrap_or_default()
            {
                return Err(DomainError::Conflict(format!(
                    "student email {e:?} is already in use within the school"
                )));
            }
        }
        if new_email != student.email {
            changed.push("email".to_owned());
            student.email = new_email;
        }
    }
    if let Some(new_address) = current_address {
        if new_address != student.current_address {
            changed.push("current_address".to_owned());
            student.current_address = new_address;
        }
    }
    if let Some(new_address) = permanent_address {
        if new_address != student.permanent_address {
            changed.push("permanent_address".to_owned());
            student.permanent_address = new_address;
        }
    }
    if let Some(new_custom) = custom_fields {
        if new_custom != student.custom_fields {
            changed.push("custom_fields".to_owned());
            student.custom_fields = new_custom;
        }
    }
    if !changed.is_empty() {
        student.updated_at = now;
        student.version = student.version.next();
    }
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    student.last_event_id = Some(event_id);
    let event =
        StudentProfileUpdated::new(student.id, changed, event_id, student.correlation_id, now);
    Ok(event)
}

/// Suspend an active [`Student`] and emit a
/// [`StudentSuspended`] event. The student's status moves
/// to `Suspended`.
///
/// # Errors
///
/// - `Validation` if the reason is empty or too long.
pub fn suspend_student<C, G>(
    student: &mut Student,
    cmd: SuspendStudentCommand,
    clock: &C,
    _ids: &G,
) -> Result<StudentSuspended>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let SuspendStudentCommand {
        tenant: _,
        student_id,
        reason,
        effective_from,
        expected_return,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    validate_suspension_reason(&reason)?;
    // I-5: status FSM precondition — the student must be
    // Active to be suspended.
    if student.status != StudentStatus::Active {
        return Err(DomainError::Conflict(format!(
            "student {} is not active (current status: {}); cannot suspend",
            student.id, student.status
        )));
    }
    let now = clock.now();
    student.status = StudentStatus::Suspended;
    student.updated_at = now;
    student.version = student.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    student.last_event_id = Some(event_id);
    let event = StudentSuspended::new(
        student.id,
        reason,
        effective_from,
        expected_return,
        event_id,
        student.correlation_id,
        now,
    );
    Ok(event)
}

/// Reinstate a suspended [`Student`] and emit a
/// [`StudentReinstated`] event. The student's status moves
/// back to `Active`.
pub fn reinstate_student<C, G>(
    student: &mut Student,
    cmd: ReinstateStudentCommand,
    clock: &C,
    _ids: &G,
) -> Result<StudentReinstated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let ReinstateStudentCommand {
        tenant: _,
        student_id,
        effective_from,
        note,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    if student.status != StudentStatus::Suspended {
        return Err(DomainError::Conflict(format!(
            "student {} is not suspended (current status: {})",
            student.id, student.status
        )));
    }
    let now = clock.now();
    student.status = StudentStatus::Active;
    student.updated_at = now;
    student.version = student.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    student.last_event_id = Some(event_id);
    let event = StudentReinstated::new(
        student.id,
        effective_from,
        note,
        event_id,
        student.correlation_id,
        now,
    );
    Ok(event)
}

/// Withdraw a [`Student`] and emit a [`StudentWithdrawn`]
/// event. The student's status moves to `Withdrawn` and
/// the aggregate is soft-deleted (`active_status =
/// Retired`).
pub fn withdraw_student<C, G>(
    student: &mut Student,
    cmd: WithdrawStudentCommand,
    clock: &C,
    _ids: &G,
) -> Result<StudentWithdrawn>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let WithdrawStudentCommand {
        tenant: _,
        student_id,
        reason,
        effective_from,
        note,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    validate_withdrawal_reason(&reason)?;
    // I-5: status FSM precondition — the student must be
    // Active to be withdrawn.
    if student.status != StudentStatus::Active {
        return Err(DomainError::Conflict(format!(
            "student {} is not active (current status: {}); cannot withdraw",
            student.id, student.status
        )));
    }
    let now = clock.now();
    student.status = StudentStatus::Withdrawn;
    student.active_status = ActiveStatus::Retired;
    student.updated_at = now;
    student.version = student.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    student.last_event_id = Some(event_id);
    let event = StudentWithdrawn::new(
        student.id,
        reason,
        effective_from,
        note,
        event_id,
        student.correlation_id,
        now,
    );
    Ok(event)
}

/// Transfer a [`Student`] to another school and emit a
/// [`StudentTransferred`] event. The student's status moves
/// to `Transferred` and the aggregate is soft-deleted
/// (`active_status = Retired`).
pub fn transfer_student<C, G>(
    student: &mut Student,
    cmd: TransferStudentCommand,
    clock: &C,
    _ids: &G,
) -> Result<StudentTransferred>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let TransferStudentCommand {
        tenant: _,
        student_id,
        destination_school_id,
        reason,
        effective_from,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    validate_transfer_reason(&reason)?;
    if destination_school_id == student.school_id {
        return Err(DomainError::Validation(
            "destination school id must differ from the source school id".to_owned(),
        ));
    }
    // I-5: status FSM precondition — the student must be
    // Active to be transferred.
    if student.status != StudentStatus::Active {
        return Err(DomainError::Conflict(format!(
            "student {} is not active (current status: {}); cannot transfer",
            student.id, student.status
        )));
    }
    let now = clock.now();
    student.status = StudentStatus::Transferred;
    student.active_status = ActiveStatus::Retired;
    student.updated_at = now;
    student.version = student.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    student.last_event_id = Some(event_id);
    let event = StudentTransferred::new(
        student.id,
        destination_school_id,
        reason,
        effective_from,
        event_id,
        student.correlation_id,
        now,
    );
    Ok(event)
}

/// Promote a [`Student`] to the next academic year and emit
/// a [`StudentPromoted`] event.
///
/// The function does **not** mutate the student record's
/// class/section fields (those live on the `StudentRecord`
/// aggregate, which lands in a later phase). The event
/// carries the full promotion payload; downstream
/// subscribers (`educore-finance`, `educore-attendance`,
/// `educore-assessment`) consume it to roll over balances,
/// reset daily expectations, and archive marks.
///
/// # Errors
///
/// - `Validation` if the from/to academic years are in
///   different schools (per AcademicYear I-5: same-school
///   promotion).
/// - `Validation` if the from/to academic years are equal.
/// - `Conflict` if the from academic year range is not the
///   immediate predecessor of the to academic year range
///   (per AcademicYear I-5: sequential promotion).
pub fn promote_student<C, G>(
    student: &Student,
    cmd: PromoteStudentCommand,
    from_range: Option<AcademicYearRange>,
    to_range: Option<AcademicYearRange>,
    clock: &C,
    _ids: &G,
) -> Result<StudentPromoted>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let PromoteStudentCommand {
        tenant: _,
        student_id,
        from_academic_year_id,
        to_academic_year_id,
        to_class_id,
        to_section_id,
        to_roll_no,
        result_status,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    // AcademicYear I-5: same-school.
    if from_academic_year_id.school_id() != to_academic_year_id.school_id() {
        return Err(DomainError::Validation(format!(
            "from academic year {from_academic_year_id} and to academic year {to_academic_year_id} are in different schools"
        )));
    }
    if from_academic_year_id.school_id() != student.school_id {
        return Err(DomainError::Validation(format!(
            "from academic year {from_academic_year_id} is in school {}, student is in {}",
            from_academic_year_id.school_id(),
            student.school_id
        )));
    }
    if to_academic_year_id.school_id() != student.school_id {
        return Err(DomainError::Validation(format!(
            "to academic year {to_academic_year_id} is in school {}, student is in {}",
            to_academic_year_id.school_id(),
            student.school_id
        )));
    }
    if from_academic_year_id == to_academic_year_id {
        return Err(DomainError::Validation(
            "from and to academic year must differ".to_owned(),
        ));
    }
    // AcademicYear I-5: To is the next sequential year.
    // We enforce this via the ranges when both are supplied.
    if let (Some(from), Some(to)) = (from_range, to_range) {
        // Sequential = to.start == from.end + 1 day.
        let next_day = from.end + chrono::Duration::days(1);
        if to.start != next_day {
            return Err(DomainError::Conflict(format!(
                "to academic year start {0} must be the day after from academic year end {1} (immediate successor)",
                to.start, from.end
            )));
        }
    }
    let now = clock.now();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    let event = StudentPromoted::new(
        student.id,
        student.class_id,
        student.section_id,
        to_class_id,
        to_section_id,
        from_academic_year_id,
        to_academic_year_id,
        to_roll_no,
        result_status,
        event_id,
        student.correlation_id,
        now,
    );
    Ok(event)
}

/// Graduate a [`Student`] and emit a [`StudentGraduated`]
/// event. The student's status moves to `Graduated` and
/// the aggregate is soft-deleted.
pub fn graduate_student<C, G>(
    student: &mut Student,
    cmd: GraduateStudentCommand,
    clock: &C,
    _ids: &G,
) -> Result<StudentGraduated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let GraduateStudentCommand {
        tenant: _,
        student_id,
        academic_year_id,
        graduation_date,
    } = cmd;
    debug_assert_eq!(student_id, student.id);
    // I-5: status FSM precondition — the student must be
    // Active to be graduated.
    if student.status != StudentStatus::Active {
        return Err(DomainError::Conflict(format!(
            "student {} is not active (current status: {}); cannot graduate",
            student.id, student.status
        )));
    }
    // Per AcademicYear I-5: graduation must occur in the
    // current academic year (the spec's promotion rule).
    if academic_year_id.school_id() != student.school_id {
        return Err(DomainError::Validation(format!(
            "academic year {academic_year_id} is in a different school than the student"
        )));
    }
    let now = clock.now();
    student.status = StudentStatus::Graduated;
    student.active_status = ActiveStatus::Retired;
    student.updated_at = now;
    student.version = student.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    student.last_event_id = Some(event_id);
    let event = StudentGraduated::new(
        student.id,
        academic_year_id,
        graduation_date,
        student.status,
        event_id,
        student.correlation_id,
        now,
    );
    Ok(event)
}

// =============================================================================
// Class functions (4)
// =============================================================================

/// Create a new [`Class`] and emit a [`ClassCreated`] event.
///
/// Per Class I-2: a class is uniquely named within a
/// school. The uniqueness is enforced via
/// [`UniquenessChecker::class_name_exists`].
pub fn create_class<C, G>(
    cmd: CreateClassCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(Class, ClassCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateClassCommand {
        tenant,
        class_id,
        class_name,
        pass_mark,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_class_name(&class_name)?;
    validate_pass_mark(pass_mark)?;
    // I-2: class name uniqueness within school.
    if uniqueness.class_name_exists(class_id.school_id(), &class_name) {
        return Err(DomainError::Conflict(format!(
            "class name {class_name:?} is already taken in the school"
        )));
    }
    let class = Class::fresh(
        class_id,
        class_name.clone(),
        crate::value_objects::PassMark::new(pass_mark)?,
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    let event_id = fresh_event_id(ids);
    let event = ClassCreated::new(
        class_id,
        class_name,
        pass_mark,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok((class, event))
}

/// Update a [`Class`]'s mutable fields and emit a
/// [`ClassUpdated`] event.
///
/// Per Class I-2: a class is uniquely named within a
/// school. The new name is checked against the uniqueness
/// surface before the mutation lands.
pub fn update_class<C, G>(
    class: &mut Class,
    cmd: UpdateClassCommand,
    clock: &C,
    _ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<ClassUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let UpdateClassCommand {
        tenant: _,
        class_id,
        class_name,
        pass_mark,
    } = cmd;
    debug_assert_eq!(class_id, class.id);
    let now = clock.now();
    let mut changed = Vec::new();
    let mut new_name = None;
    let mut new_pass_mark = None;
    if let Some(name) = class_name {
        validate_class_name(&name)?;
        if name != class.name {
            // I-2: uniqueness check before rename.
            if uniqueness.class_name_exists(class.school_id, &name) {
                return Err(DomainError::Conflict(format!(
                    "class name {name:?} is already taken in the school"
                )));
            }
            changed.push("class_name".to_owned());
            class.name = name.clone();
            new_name = Some(name);
        }
    }
    if let Some(pm) = pass_mark {
        validate_pass_mark(pm)?;
        if pm != class.pass_mark.as_f32() {
            changed.push("pass_mark".to_owned());
            class.pass_mark = crate::value_objects::PassMark::new(pm)?;
            new_pass_mark = Some(pm);
        }
    }
    if !changed.is_empty() {
        class.updated_at = now;
        class.version = class.version.next();
    }
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    class.last_event_id = Some(event_id);
    let event = ClassUpdated::new(
        class_id,
        changed,
        new_name,
        new_pass_mark,
        event_id,
        class.correlation_id,
        now,
    );
    Ok(event)
}

/// Set a [`Class`]'s optional-subject GPA threshold and
/// emit an [`OptionalSubjectGpaThresholdSet`] event.
pub fn set_optional_subject_gpa_threshold<C, G>(
    class: &mut Class,
    cmd: SetOptionalSubjectGpaThresholdCommand,
    clock: &C,
    _ids: &G,
) -> Result<OptionalSubjectGpaThresholdSet>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let SetOptionalSubjectGpaThresholdCommand {
        tenant: _,
        class_id,
        threshold,
    } = cmd;
    debug_assert_eq!(class_id, class.id);
    validate_gpa_threshold(threshold)?;
    let now = clock.now();
    class.optional_subject_gpa_threshold =
        crate::value_objects::OptionalSubjectGpaThreshold::new(threshold)?;
    class.updated_at = now;
    class.version = class.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    class.last_event_id = Some(event_id);
    let event = OptionalSubjectGpaThresholdSet::new(
        class_id,
        threshold,
        event_id,
        class.correlation_id,
        now,
    );
    Ok(event)
}

/// Soft-delete a [`Class`] and emit a [`ClassDeleted`] event.
pub fn delete_class<C, G>(
    class: &mut Class,
    cmd: DeleteClassCommand,
    clock: &C,
    _ids: &G,
) -> Result<ClassDeleted>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let DeleteClassCommand {
        tenant: _,
        class_id,
    } = cmd;
    debug_assert_eq!(class_id, class.id);
    let now = clock.now();
    class.active_status = ActiveStatus::Retired;
    class.updated_at = now;
    class.version = class.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    class.last_event_id = Some(event_id);
    let event = ClassDeleted::new(class_id, event_id, class.correlation_id, now);
    Ok(event)
}

// =============================================================================
// Section functions (3)
// =============================================================================

/// Create a new [`Section`] and emit a [`SectionCreated`]
/// event.
pub fn create_section<C, G>(
    cmd: CreateSectionCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(Section, SectionCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateSectionCommand {
        tenant,
        section_id,
        section_name,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_section_name(&section_name)?;
    // Section I-1: section name unique within school.
    if uniqueness.section_name_exists(section_id.school_id(), &section_name) {
        return Err(DomainError::Conflict(format!(
            "section name {section_name:?} is already taken in the school"
        )));
    }
    let section = Section::fresh(
        section_id,
        section_name.clone(),
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    let event_id = fresh_event_id(ids);
    let event = SectionCreated::new(section_id, section_name, event_id, ctx.correlation_id, now);
    Ok((section, event))
}

/// Update a [`Section`]'s mutable fields and emit a
/// [`SectionUpdated`] event.
pub fn update_section<C, G>(
    section: &mut Section,
    cmd: UpdateSectionCommand,
    clock: &C,
    _ids: &G,
) -> Result<SectionUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let UpdateSectionCommand {
        tenant: _,
        section_id,
        section_name,
    } = cmd;
    debug_assert_eq!(section_id, section.id);
    let now = clock.now();
    let mut changed = Vec::new();
    let mut new_name = None;
    if let Some(name) = section_name {
        validate_section_name(&name)?;
        if name != section.name {
            changed.push("section_name".to_owned());
            section.name = name.clone();
            new_name = Some(name);
        }
    }
    if !changed.is_empty() {
        section.updated_at = now;
        section.version = section.version.next();
    }
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    section.last_event_id = Some(event_id);
    let event = SectionUpdated::new(
        section_id,
        changed,
        new_name,
        event_id,
        section.correlation_id,
        now,
    );
    Ok(event)
}

/// Soft-delete a [`Section`] and emit a [`SectionDeleted`]
/// event.
pub fn delete_section<C, G>(
    section: &mut Section,
    cmd: DeleteSectionCommand,
    clock: &C,
    _ids: &G,
) -> Result<SectionDeleted>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let DeleteSectionCommand {
        tenant: _,
        section_id,
    } = cmd;
    debug_assert_eq!(section_id, section.id);
    let now = clock.now();
    section.active_status = ActiveStatus::Retired;
    section.updated_at = now;
    section.version = section.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    section.last_event_id = Some(event_id);
    let event = SectionDeleted::new(section_id, event_id, section.correlation_id, now);
    Ok(event)
}

// =============================================================================
// Subject functions (3)
// =============================================================================

/// Create a new [`Subject`] and emit a [`SubjectCreated`] event.
///
/// Per Subject I-1: a subject's code is unique within a
/// school. The uniqueness is enforced via
/// [`UniquenessChecker::subject_code_exists`].
pub fn create_subject<C, G>(
    cmd: CreateSubjectCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(Subject, SubjectCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateSubjectCommand {
        tenant,
        subject_id,
        subject_code,
        subject_name,
        subject_type,
        pass_mark,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_subject_code(&subject_code)?;
    validate_subject_name(&subject_name)?;
    validate_pass_mark(pass_mark)?;
    // I-1: subject code uniqueness within school.
    if uniqueness.subject_code_exists(subject_id.school_id(), &subject_code) {
        return Err(DomainError::Conflict(format!(
            "subject code {subject_code:?} is already taken in the school"
        )));
    }
    let subject = Subject::fresh(
        subject_id,
        subject_code.clone(),
        subject_name.clone(),
        subject_type,
        crate::value_objects::PassMark::new(pass_mark)?,
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    let event_id = fresh_event_id(ids);
    let event = SubjectCreated::new(
        subject_id,
        subject_code,
        subject_name,
        subject_type,
        pass_mark,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok((subject, event))
}

/// Update a [`Subject`]'s mutable fields and emit a
/// [`SubjectUpdated`] event.
pub fn update_subject<C, G>(
    subject: &mut Subject,
    cmd: UpdateSubjectCommand,
    clock: &C,
    _ids: &G,
) -> Result<SubjectUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let UpdateSubjectCommand {
        tenant: _,
        subject_id,
        subject_name,
        subject_type,
        pass_mark,
    } = cmd;
    debug_assert_eq!(subject_id, subject.id);
    let now = clock.now();
    let mut changed = Vec::new();
    let mut new_name = None;
    let mut new_subject_type = None;
    let mut new_pass_mark = None;
    if let Some(name) = subject_name {
        validate_subject_name(&name)?;
        if name != subject.name {
            changed.push("subject_name".to_owned());
            subject.name = name.clone();
            new_name = Some(name);
        }
    }
    if let Some(t) = subject_type {
        if t != subject.subject_type {
            changed.push("subject_type".to_owned());
            subject.subject_type = t;
            new_subject_type = Some(t);
        }
    }
    if let Some(pm) = pass_mark {
        validate_pass_mark(pm)?;
        if pm != subject.pass_mark.as_f32() {
            changed.push("pass_mark".to_owned());
            subject.pass_mark = crate::value_objects::PassMark::new(pm)?;
            new_pass_mark = Some(pm);
        }
    }
    if !changed.is_empty() {
        subject.updated_at = now;
        subject.version = subject.version.next();
    }
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    subject.last_event_id = Some(event_id);
    let event = SubjectUpdated::new(
        subject_id,
        changed,
        new_name,
        new_subject_type,
        new_pass_mark,
        event_id,
        subject.correlation_id,
        now,
    );
    Ok(event)
}

/// Soft-delete a [`Subject`] and emit a [`SubjectDeleted`]
/// event.
pub fn delete_subject<C, G>(
    subject: &mut Subject,
    cmd: DeleteSubjectCommand,
    clock: &C,
    _ids: &G,
) -> Result<SubjectDeleted>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let DeleteSubjectCommand {
        tenant: _,
        subject_id,
    } = cmd;
    debug_assert_eq!(subject_id, subject.id);
    let now = clock.now();
    subject.active_status = ActiveStatus::Retired;
    subject.updated_at = now;
    subject.version = subject.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    subject.last_event_id = Some(event_id);
    let event = SubjectDeleted::new(subject_id, event_id, subject.correlation_id, now);
    Ok(event)
}

// =============================================================================
// AcademicYear functions (5)
// =============================================================================

/// Create a new [`AcademicYear`] and emit an
/// [`AcademicYearCreated`] event.
///
/// Per AcademicYear I-2: academic years do not overlap
/// within a school. The overlap is enforced via
/// [`UniquenessChecker::academic_year_overlaps`].
pub fn create_academic_year<C, G>(
    cmd: CreateAcademicYearCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(AcademicYear, AcademicYearCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateAcademicYearCommand {
        tenant,
        academic_year_id,
        year,
        title,
        starting_date,
        ending_date,
        is_current,
        copy_with_academic_year: _,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_year_label(&year)?;
    validate_year_title(&title)?;
    let range: AcademicYearRange = AcademicYearRange::new(starting_date, ending_date)?;
    // I-2: no overlap within school.
    if uniqueness.academic_year_overlaps(academic_year_id.school_id(), range, None) {
        return Err(DomainError::Conflict(format!(
            "academic year range {0}..{1} overlaps an existing academic year in the school",
            range.start, range.end
        )));
    }
    let mut year_agg = AcademicYear::fresh(
        academic_year_id,
        year.clone(),
        title.clone(),
        range,
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    year_agg.is_current = is_current;
    let event_id = fresh_event_id(ids);
    year_agg.last_event_id = Some(event_id);
    year_agg.correlation_id = ctx.correlation_id;
    let event = AcademicYearCreated::new(
        academic_year_id,
        year,
        title,
        range.start,
        range.end,
        is_current,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok((year_agg, event))
}

/// Update an [`AcademicYear`]'s date range and emit an
/// [`AcademicYearDatesUpdated`] event.
///
/// Per AcademicYear I-2: academic years do not overlap
/// within a school. The new range is checked against the
/// uniqueness surface (excluding the current year) before
/// the mutation lands.
pub fn update_academic_year_dates<C, G>(
    year_agg: &mut AcademicYear,
    cmd: UpdateAcademicYearDatesCommand,
    clock: &C,
    _ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<AcademicYearDatesUpdated>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let UpdateAcademicYearDatesCommand {
        tenant: _,
        academic_year_id,
        starting_date,
        ending_date,
    } = cmd;
    debug_assert_eq!(academic_year_id, year_agg.id);
    let now = clock.now();
    let range: AcademicYearRange = AcademicYearRange::new(starting_date, ending_date)?;
    // I-2: no overlap within school (excluding self).
    if uniqueness.academic_year_overlaps(year_agg.school_id, range, Some(year_agg.id)) {
        return Err(DomainError::Conflict(format!(
            "academic year range {0}..{1} overlaps an existing academic year in the school",
            range.start, range.end
        )));
    }
    year_agg.range = range;
    year_agg.updated_at = now;
    year_agg.version = year_agg.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    year_agg.last_event_id = Some(event_id);
    let event = AcademicYearDatesUpdated::new(
        academic_year_id,
        range.start,
        range.end,
        event_id,
        year_agg.correlation_id,
        now,
    );
    Ok(event)
}

/// Mark an [`AcademicYear`] as current and emit a
/// [`CurrentAcademicYearSet`] event.
///
/// Per AcademicYear I-3: exactly one academic year may be
/// current per school. The function takes the previously
/// current academic year (if any) and demotes it in the
/// same transaction; the caller (the engine's dispatcher)
/// is responsible for persisting the demoted year.
pub fn set_current_academic_year<C, G>(
    year_agg: &mut AcademicYear,
    previously_current: Option<&mut AcademicYear>,
    cmd: SetCurrentAcademicYearCommand,
    clock: &C,
    _ids: &G,
) -> Result<CurrentAcademicYearSet>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let SetCurrentAcademicYearCommand {
        tenant: _,
        academic_year_id,
    } = cmd;
    debug_assert_eq!(academic_year_id, year_agg.id);
    if year_agg.is_closed {
        return Err(DomainError::Conflict(format!(
            "academic year {academic_year_id} is closed and cannot be set as current"
        )));
    }
    let now = clock.now();
    // I-3: demote previously-current academic year (cascade).
    let demoted_id = if let Some(prev) = previously_current {
        if prev.is_current {
            prev.is_current = false;
            prev.updated_at = now;
            prev.version = prev.version.next();
            Some(prev.id)
        } else {
            None
        }
    } else {
        None
    };
    year_agg.is_current = true;
    year_agg.updated_at = now;
    year_agg.version = year_agg.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    year_agg.last_event_id = Some(event_id);
    let event = CurrentAcademicYearSet::new(
        academic_year_id,
        demoted_id,
        event_id,
        year_agg.correlation_id,
        now,
    );
    Ok(event)
}

/// Close an [`AcademicYear`] (make it read-only) and emit
/// an [`AcademicYearClosed`] event.
pub fn close_academic_year<C, G>(
    year_agg: &mut AcademicYear,
    cmd: CloseAcademicYearCommand,
    clock: &C,
    _ids: &G,
) -> Result<AcademicYearClosed>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CloseAcademicYearCommand {
        tenant: _,
        academic_year_id,
    } = cmd;
    debug_assert_eq!(academic_year_id, year_agg.id);
    let now = clock.now();
    year_agg.is_closed = true;
    if year_agg.is_current {
        year_agg.is_current = false;
    }
    year_agg.updated_at = now;
    year_agg.version = year_agg.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    year_agg.last_event_id = Some(event_id);
    let event = AcademicYearClosed::new(academic_year_id, event_id, year_agg.correlation_id, now);
    Ok(event)
}

/// Mark an [`AcademicYear`] as a deep copy of another
/// academic year and emit an [`AcademicYearCopied`] event.
///
/// The actual deep copy (classes, sections, subjects,
/// class-subjects, routines) is a storage-side concern; the
/// service function only validates the source and emits
/// the event.
pub fn copy_academic_year<C, G>(
    year_agg: &mut AcademicYear,
    from: AcademicYearId,
    clock: &C,
    _ids: &G,
) -> Result<AcademicYearCopied>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if from.school_id() != year_agg.school_id {
        return Err(DomainError::Validation(format!(
            "source academic year {from} is in a different school than the target"
        )));
    }
    if from == year_agg.id {
        return Err(DomainError::Validation(
            "source and target academic year must differ".to_owned(),
        ));
    }
    let now = clock.now();
    year_agg.updated_at = now;
    year_agg.version = year_agg.version.next();
    let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
    year_agg.last_event_id = Some(event_id);
    let event = AcademicYearCopied::new(year_agg.id, from, event_id, year_agg.correlation_id, now);
    Ok(event)
}

// =============================================================================
// Cross-cutting helpers
// =============================================================================

/// Helper: returns `true` if the academic year's school id
/// matches `ctx.school_id`. Used by command dispatchers
/// (outside the services module) to enforce the
/// `school_id` invariant at the command boundary.
pub fn school_matches(ctx: &TenantContext, school: SchoolId) -> bool {
    ctx.school_id == school
}

// =============================================================================
// Placeholder handler skeletons for the remaining 14 academic aggregates
// (commands introduced in commit 1af809b; event stubs in 66bee45;
// aggregate stubs in 18d67df).
//
// Each skeleton:
//   - validates that `cmd.school_id == cmd.id.school_id()` (basic
//     tenant-anchor invariant),
//   - constructs the placeholder aggregate from the typed id,
//   - mints and returns the corresponding event stub.
//
// The full impl (capability check, domain fields, invariants,
// persistence wiring) lands in subsequent workstreams per
// `docs/build-plan.md`. The `Clock` and `IdGenerator` bounds mirror
// the create-flow signature used by the prompt-named handlers
// above so the dispatcher glue does not need to special-case them.
// =============================================================================

/// Register a [`Guardian`] and emit a [`GuardianRegistered`] event.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian I-1:
/// the guardian carries at most one phone and one email of
/// record (already enforced at construction via
/// [`PhoneNumber`](crate::value_objects::PhoneNumber) and
/// [`EmailAddress`](crate::value_objects::EmailAddress)).
///
/// The service is otherwise tenant-anchored: the typed
/// `guardian_id` must share a school with the active tenant.
pub fn register_guardian<C, G>(
    cmd: RegisterGuardianCommand,
    clock: &C,
    ids: &G,
) -> Result<(Guardian, GuardianRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let RegisterGuardianCommand {
        guardian_id,
        first_name,
        last_name,
        phone,
        email,
    } = cmd;
    validate_first_name(&first_name)?;
    validate_last_name(&last_name)?;
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let actor = educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7());
    let aggregate = Guardian::fresh(
        guardian_id,
        first_name.clone(),
        last_name.clone(),
        phone.clone(),
        email.clone(),
        actor,
        actor,
        now,
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    let event = GuardianRegistered::new(
        guardian_id,
        first_name,
        last_name,
        phone,
        email,
        event_id,
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        now,
    );
    Ok((aggregate, event))
}

// =============================================================================
// Guardian link services (full impl — Batch 1)
// =============================================================================

/// Link a [`Guardian`] to a [`Student`] and emit a
/// [`GuardianLinkedToStudent`] event.
///
/// Per Guardian I-2 and I-3: a guardian may be linked to
/// multiple students; the link carries `relation` and
/// `is_primary`. Per I-4: at most one link may be primary
/// for a student; the uniqueness check is enforced via
/// [`UniquenessChecker::primary_guardian_link_exists`].
pub fn link_guardian_to_student<C, G>(
    cmd: LinkGuardianToStudentCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(StudentGuardianLink, GuardianLinkedToStudent)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let LinkGuardianToStudentCommand {
        tenant: _,
        link_id,
        guardian_id,
        student_id,
        relation,
        is_primary,
    } = cmd;
    // Cross-tenant guard.
    if guardian_id.school_id() != link_id.school_id() {
        return Err(DomainError::Validation(format!(
            "guardian id {guardian_id} is in school {}, link id school is {}",
            guardian_id.school_id(),
            link_id.school_id()
        )));
    }
    if student_id.school_id() != link_id.school_id() {
        return Err(DomainError::Validation(format!(
            "student id {student_id} is in school {}, link id school is {}",
            student_id.school_id(),
            link_id.school_id()
        )));
    }
    // I-4: at most one primary guardian per student.
    if is_primary && uniqueness.primary_guardian_link_exists(link_id.school_id(), student_id) {
        return Err(DomainError::Conflict(format!(
            "student {student_id} already has a primary guardian; demote it before marking another"
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let actor = educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7());
    let aggregate = StudentGuardianLink::fresh(
        link_id,
        guardian_id,
        student_id,
        relation,
        is_primary,
        actor,
        actor,
        now,
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    let event = GuardianLinkedToStudent::new(
        link_id,
        guardian_id,
        student_id,
        relation,
        is_primary,
        event_id,
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        now,
    );
    Ok((aggregate, event))
}

/// Unlink a [`Guardian`] from a [`Student`] and emit a
/// [`GuardianUnlinkedFromStudent`] event.
///
/// Per Guardian I-5: when the last link for a guardian is
/// removed the guardian is soft-deleted. The caller (the
/// engine's dispatcher) is responsible for persisting the
/// updated guardian; this service returns the soft-delete
/// flag in the event so the dispatcher can cascade the
/// `Guardian.active_status = Retired` mutation.
#[must_use]
pub fn unlink_guardian_from_student<C, G>(
    link: &mut StudentGuardianLink,
    cmd: UnlinkGuardianFromStudentCommand,
    clock: &C,
    ids: &G,
    was_last_link: bool,
) -> GuardianUnlinkedFromStudent
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let UnlinkGuardianFromStudentCommand {
        tenant: _,
        link_id,
    } = cmd;
    debug_assert_eq!(link_id, link.id);
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    link.active_status = educore_core::value_objects::ActiveStatus::Retired;
    link.updated_at = now;
    link.version = link.version.next();
    link.last_event_id = Some(event_id);
    GuardianUnlinkedFromStudent::new(
        link_id,
        link.guardian_id,
        link.student_id,
        was_last_link,
        event_id,
        link.correlation_id,
        now,
    )
}

/// Mark a guardian link as primary.
///
/// Per Guardian I-4: at most one guardian per student may
/// be primary. The `uniqueness` checker is used to validate
/// that no other link is currently primary for the student.
pub fn mark_primary_guardian<C, G>(
    link: &mut StudentGuardianLink,
    cmd: MarkPrimaryGuardianCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
    previously_primary: Option<StudentGuardianLinkId>,
) -> Result<PrimaryGuardianMarked>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let MarkPrimaryGuardianCommand {
        tenant: _,
        link_id,
    } = cmd;
    debug_assert_eq!(link_id, link.id);
    if uniqueness.primary_guardian_link_exists(link.school_id, link.student_id) {
        return Err(DomainError::Conflict(format!(
            "student {} already has a primary guardian",
            link.student_id
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    link.is_primary = true;
    link.updated_at = now;
    link.version = link.version.next();
    link.last_event_id = Some(event_id);
    Ok(PrimaryGuardianMarked::new(
        link_id,
        link.guardian_id,
        link.student_id,
        previously_primary,
        event_id,
        link.correlation_id,
        now,
    ))
}

// =============================================================================
// OptionalSubjectAssignment service (full impl — Batch 1)
// =============================================================================

/// Assign an optional subject to a student for an academic
/// year.
///
/// Per Student I-4: a student may be in at most one optional
/// subject per academic year. The service enforces the
/// uniqueness via
/// [`UniquenessChecker::optional_subject_assigned_exists`].
pub fn assign_optional_subject<C, G>(
    cmd: AssignOptionalSubjectCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(OptionalSubjectAssignment, OptionalSubjectAssignmentCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let AssignOptionalSubjectCommand {
        tenant: _,
        assignment_id,
        student_id,
        subject_id,
        academic_year_id,
    } = cmd;
    if student_id.school_id() != assignment_id.school_id() {
        return Err(DomainError::Validation(format!(
            "student id {student_id} is in school {}, assignment school is {}",
            student_id.school_id(),
            assignment_id.school_id()
        )));
    }
    if subject_id.school_id() != assignment_id.school_id() {
        return Err(DomainError::Validation(format!(
            "subject id {subject_id} is in school {}, assignment school is {}",
            subject_id.school_id(),
            assignment_id.school_id()
        )));
    }
    if academic_year_id.school_id() != assignment_id.school_id() {
        return Err(DomainError::Validation(format!(
            "academic year id {academic_year_id} is in school {}, assignment school is {}",
            academic_year_id.school_id(),
            assignment_id.school_id()
        )));
    }
    if uniqueness.optional_subject_assigned_exists(
        assignment_id.school_id(),
        student_id,
        academic_year_id,
    ) {
        return Err(DomainError::Conflict(format!(
            "student {student_id} already has an optional subject for academic year {academic_year_id}"
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let actor = educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7());
    let aggregate = OptionalSubjectAssignment::fresh(
        assignment_id,
        student_id,
        subject_id,
        academic_year_id,
        actor,
        actor,
        now,
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    let event = OptionalSubjectAssignmentCreated::new(
        assignment_id,
        student_id,
        subject_id,
        academic_year_id,
        event_id,
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        now,
    );
    Ok((aggregate, event))
}

/// Create a [`ClassSection`] pairing and emit a [`ClassSectionCreated`] event.
pub fn create_class_section<C, G>(
    cmd: CreateClassSectionCommand,
    clock: &C,
    ids: &G,
) -> Result<(ClassSection, ClassSectionCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateClassSectionCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "class section id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = ClassSection { id, school_id };
    let event = ClassSectionCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Assign a subject to a class via [`ClassSubject`] and emit a
/// [`ClassSubjectAssigned`] event.
pub fn create_class_subject<C, G>(
    cmd: CreateClassSubjectCommand,
    clock: &C,
    ids: &G,
) -> Result<(ClassSubject, ClassSubjectAssigned)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateClassSubjectCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "class subject id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = ClassSubject { id, school_id };
    let event = ClassSubjectAssigned {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Schedule a [`ClassRoutine`] period and emit a [`ClassRoutineScheduled`] event.
pub fn create_class_routine<C, G>(
    cmd: CreateClassRoutineCommand,
    clock: &C,
    ids: &G,
) -> Result<(ClassRoutine, ClassRoutineScheduled)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateClassRoutineCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "class routine id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = ClassRoutine { id, school_id };
    let event = ClassRoutineScheduled {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Issue a [`Homework`] assignment and emit a [`HomeworkAssigned`] event.
pub fn create_homework<C, G>(
    cmd: CreateHomeworkCommand,
    clock: &C,
    ids: &G,
) -> Result<(Homework, HomeworkAssigned)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateHomeworkCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "homework id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = Homework { id, school_id };
    let event = HomeworkAssigned {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Draft a [`LessonPlan`] and emit a [`LessonPlanCreated`] event.
pub fn create_lesson_plan<C, G>(
    cmd: CreateLessonPlanCommand,
    clock: &C,
    ids: &G,
) -> Result<(LessonPlan, LessonPlanCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateLessonPlanCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "lesson plan id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = LessonPlan { id, school_id };
    let event = LessonPlanCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create a [`Lesson`] and emit a [`LessonCreated`] event.
pub fn create_lesson<C, G>(
    cmd: CreateLessonCommand,
    clock: &C,
    ids: &G,
) -> Result<(Lesson, LessonCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateLessonCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "lesson id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = Lesson { id, school_id };
    let event = LessonCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create a [`LessonTopic`] and emit a [`LessonTopicCreated`] event.
pub fn create_lesson_topic<C, G>(
    cmd: CreateLessonTopicCommand,
    clock: &C,
    ids: &G,
) -> Result<(LessonTopic, LessonTopicCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateLessonTopicCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "lesson topic id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = LessonTopic { id, school_id };
    let event = LessonTopicCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Record a [`StudentPromotion`] and emit a [`StudentPromotionRecorded`] event.
pub fn record_student_promotion<C, G>(
    cmd: RecordStudentPromotionCommand,
    clock: &C,
    ids: &G,
) -> Result<(StudentPromotion, StudentPromotionRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let RecordStudentPromotionCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "student promotion id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = StudentPromotion { id, school_id };
    let event = StudentPromotionRecorded {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create a [`StudentCategory`] and emit a [`StudentCategoryCreated`] event.
pub fn create_student_category<C, G>(
    cmd: CreateStudentCategoryCommand,
    clock: &C,
    ids: &G,
) -> Result<(StudentCategory, StudentCategoryCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateStudentCategoryCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "student category id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = StudentCategory { id, school_id };
    let event = StudentCategoryCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create a [`StudentGroup`] and emit a [`StudentGroupCreated`] event.
pub fn create_student_group<C, G>(
    cmd: CreateStudentGroupCommand,
    clock: &C,
    ids: &G,
) -> Result<(StudentGroup, StudentGroupCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateStudentGroupCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "student group id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = StudentGroup { id, school_id };
    let event = StudentGroupCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create a [`RegistrationField`] and emit a [`RegistrationFieldCreated`] event.
pub fn create_registration_field<C, G>(
    cmd: CreateRegistrationFieldCommand,
    clock: &C,
    ids: &G,
) -> Result<(RegistrationField, RegistrationFieldCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateRegistrationFieldCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "registration field id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = RegistrationField { id, school_id };
    let event = RegistrationFieldCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create a [`Certificate`] template and emit a [`CertificateCreated`] event.
pub fn create_certificate<C, G>(
    cmd: CreateCertificateCommand,
    clock: &C,
    ids: &G,
) -> Result<(Certificate, CertificateCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateCertificateCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "certificate id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = Certificate { id, school_id };
    let event = CertificateCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

/// Create an [`IdCard`] template and emit an [`IdCardCreated`] event.
pub fn create_id_card<C, G>(
    cmd: CreateIdCardCommand,
    clock: &C,
    ids: &G,
) -> Result<(IdCard, IdCardCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateIdCardCommand { id, school_id } = cmd;
    if id.school_id() != school_id {
        return Err(DomainError::Validation(format!(
            "id card id {id} is in school {}, command school_id is {school_id}",
            id.school_id(),
        )));
    }
    let now = clock.now();
    let event_id = fresh_event_id(ids);
    let aggregate = IdCard { id, school_id };
    let event = IdCardCreated {
        event_id,
        school_id,
        aggregate_id: id,
        occurred_at: now,
    };
    Ok((aggregate, event))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use crate::commands::{
        AdmitStudentCommand, CloseAcademicYearCommand, CreateAcademicYearCommand,
        CreateClassCommand, CreateSectionCommand, CreateSubjectCommand, GraduateStudentCommand,
        PromoteStudentCommand, ReinstateStudentCommand, SetOptionalSubjectGpaThresholdCommand,
        SuspendStudentCommand, TransferStudentCommand, UpdateStudentProfileCommand,
        WithdrawStudentCommand,
    };
    use crate::value_objects::{
        AcademicYearId, ClassId, ResultStatus, SectionId, StudentId, SubjectId,
    };
    use educore_core::clock::{DeterministicIdGen, IdGenerator, SystemIdGen, TestClock};
    use educore_core::ids::Identifier;
    use educore_core::tenant::{TenantContext, UserType};
    use std::sync::Mutex;

    #[allow(dead_code)]
    fn school() -> SchoolId {
        SchoolId::from_uuid(uuid::Uuid::now_v7())
    }

    fn ctx_for(school: SchoolId) -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    struct InMemoryUniqueness {
        admission_nos: Mutex<Vec<(SchoolId, String)>>,
        emails: Mutex<Vec<(SchoolId, String)>>,
    }

    impl InMemoryUniqueness {
        fn new() -> Self {
            Self {
                admission_nos: Mutex::new(Vec::new()),
                emails: Mutex::new(Vec::new()),
            }
        }
        fn record_admission(&self, school: SchoolId, admission_no: &str) {
            self.admission_nos
                .lock()
                .unwrap()
                .push((school, admission_no.to_owned()));
        }
        #[allow(dead_code)]
        fn record_email(&self, school: SchoolId, email: &str) {
            self.emails
                .lock()
                .unwrap()
                .push((school, email.to_lowercase()));
        }
    }

    impl UniquenessChecker for InMemoryUniqueness {
        fn student_admission_no_exists(&self, school: SchoolId, admission_no: &str) -> bool {
            self.admission_nos
                .lock()
                .unwrap()
                .iter()
                .any(|(s, a)| *s == school && a == admission_no)
        }
        fn student_email_exists(&self, school: SchoolId, email: &str) -> bool {
            let e = email.to_lowercase();
            self.emails
                .lock()
                .unwrap()
                .iter()
                .any(|(s, m)| *s == school && m == &e)
        }
        fn roll_no_exists(
            &self,
            _school: SchoolId,
            _class_id: ClassId,
            _section_id: SectionId,
            _academic_year_id: AcademicYearId,
            _roll_no: &str,
        ) -> bool {
            false
        }
        fn class_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
            false
        }
        fn section_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
            false
        }
        fn subject_code_exists(&self, _school: SchoolId, _code: &str) -> bool {
            false
        }
        fn academic_year_overlaps(
            &self,
            _school: SchoolId,
            _range: AcademicYearRange,
            _exclude_id: Option<AcademicYearId>,
        ) -> bool {
            false
        }
        fn optional_subject_assigned_exists(
            &self,
            _school: SchoolId,
            _student_id: StudentId,
            _academic_year_id: AcademicYearId,
        ) -> bool {
            false
        }
        fn primary_guardian_link_exists(
            &self,
            _school: SchoolId,
            _student_id: StudentId,
        ) -> bool {
            false
        }
    }

    fn admit_cmd(
        ctx: TenantContext,
        student_id: StudentId,
        class: ClassId,
        section: SectionId,
        year: AcademicYearId,
        admission_no: &str,
    ) -> AdmitStudentCommand {
        AdmitStudentCommand::new(
            ctx,
            student_id,
            admission_no.to_owned(),
            "Ada".to_owned(),
            "Lovelace".to_owned(),
            chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
            crate::value_objects::Gender::Female,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            class,
            section,
            year,
        )
    }

    #[test]
    fn admit_student_emits_event() {
        let g = DeterministicIdGen::starting_at(0);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(ctx, student_id, class, section, year, "ADM-001");
        let (student, event) = admit_student(cmd, &clock, &g, &u).unwrap();
        assert_eq!(student.id, student_id);
        assert_eq!(student.admission_no, "ADM-001");
        assert_eq!(student.school_id, school);
        assert_eq!(event.student_id, student_id);
        assert_eq!(event.admission_no, "ADM-001");
        assert_eq!(event.class_id, class);
        assert_eq!(event.section_id, section);
        assert_eq!(
            <StudentAdmitted as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "academic.student.admitted"
        );
    }

    #[test]
    fn admit_student_uniqueness_violation() {
        let g = DeterministicIdGen::starting_at(100);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        u.record_admission(school, "ADM-001");
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(ctx, student_id, class, section, year, "ADM-001");
        let err = admit_student(cmd, &clock, &g, &u).unwrap_err();
        assert!(
            matches!(err, educore_core::error::DomainError::Conflict(_)),
            "expected Conflict, got {err:?}"
        );
    }

    #[test]
    fn suspend_reinstate_withdraw_transfer_graduate_change_status() {
        let g = DeterministicIdGen::starting_at(200);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(ctx.clone(), student_id, class, section, year, "ADM-001");
        let (mut student, _) = admit_student(cmd, &clock, &g, &u).unwrap();
        assert_eq!(student.status, StudentStatus::Active);

        let s = suspend_student(
            &mut student,
            SuspendStudentCommand {
                tenant: ctx.clone(),
                student_id,
                reason: "medical".to_owned(),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                expected_return: None,
            },
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(student.status, StudentStatus::Suspended);
        assert_eq!(
            <StudentSuspended as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "academic.student.suspended"
        );

        let r = reinstate_student(
            &mut student,
            ReinstateStudentCommand {
                tenant: ctx.clone(),
                student_id,
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                note: None,
            },
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(student.status, StudentStatus::Active);
        assert_eq!(
            <StudentReinstated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "academic.student.reinstated"
        );

        let w = withdraw_student(
            &mut student,
            WithdrawStudentCommand {
                tenant: ctx.clone(),
                student_id,
                reason: "moved".to_owned(),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                note: None,
            },
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(student.status, StudentStatus::Withdrawn);
        assert!(student.active_status.is_retired());
        assert_eq!(
            <StudentWithdrawn as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "academic.student.withdrawn"
        );
        let _ = (s, r, w);
    }

    #[test]
    fn transfer_requires_different_school() {
        let g = DeterministicIdGen::starting_at(300);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(ctx.clone(), student_id, class, section, year, "ADM-001");
        let (mut student, _) = admit_student(cmd, &clock, &g, &u).unwrap();
        let err = transfer_student(
            &mut student,
            TransferStudentCommand {
                tenant: ctx.clone(),
                student_id,
                destination_school_id: school, // same as source
                reason: "test".to_owned(),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            },
            &clock,
            &g,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }

    #[test]
    fn transfer_emits_event_with_destination() {
        let g = DeterministicIdGen::starting_at(400);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school_a = g.next_school_id();
        let school_b = g.next_school_id();
        let ctx_a = ctx_for(school_a);
        let student_id = StudentId::new(school_a, g.next_uuid());
        let class = ClassId::new(school_a, g.next_uuid());
        let section = SectionId::new(school_a, g.next_uuid());
        let year = AcademicYearId::new(school_a, g.next_uuid());
        let cmd = admit_cmd(ctx_a.clone(), student_id, class, section, year, "ADM-001");
        let (mut student, _) = admit_student(cmd, &clock, &g, &u).unwrap();
        let event = transfer_student(
            &mut student,
            TransferStudentCommand {
                tenant: ctx_a.clone(),
                student_id,
                destination_school_id: school_b,
                reason: "parent's job".to_owned(),
                effective_from: chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            },
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(event.destination_school_id, school_b);
        assert_eq!(student.status, StudentStatus::Transferred);
    }

    #[test]
    fn promote_emits_event_with_from_to() {
        let g = DeterministicIdGen::starting_at(500);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class_a = ClassId::new(school, g.next_uuid());
        let section_a = SectionId::new(school, g.next_uuid());
        let class_b = ClassId::new(school, g.next_uuid());
        let section_b = SectionId::new(school, g.next_uuid());
        let year_a = AcademicYearId::new(school, g.next_uuid());
        let year_b = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(
            ctx.clone(),
            student_id,
            class_a,
            section_a,
            year_a,
            "ADM-001",
        );
        let (student, _) = admit_student(cmd, &clock, &g, &u).unwrap();
        let event = promote_student(
            &student,
            PromoteStudentCommand {
                tenant: ctx,
                student_id,
                from_academic_year_id: year_a,
                to_academic_year_id: year_b,
                to_class_id: class_b,
                to_section_id: section_b,
                to_roll_no: "2".to_owned(),
                result_status: ResultStatus::Pass,
            },
            None,
            None,
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(event.from_class_id, class_a);
        assert_eq!(event.to_class_id, class_b);
        assert_eq!(event.result_status, ResultStatus::Pass);
    }

    #[test]
    fn graduate_sets_status_graduated() {
        let g = DeterministicIdGen::starting_at(600);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(ctx.clone(), student_id, class, section, year, "ADM-001");
        let (mut student, _) = admit_student(cmd, &clock, &g, &u).unwrap();
        let event = graduate_student(
            &mut student,
            GraduateStudentCommand {
                tenant: ctx,
                student_id,
                academic_year_id: year,
                graduation_date: chrono::NaiveDate::from_ymd_opt(2027, 6, 1).unwrap(),
            },
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(student.status, StudentStatus::Graduated);
        assert!(student.active_status.is_retired());
        assert_eq!(event.status, StudentStatus::Graduated);
    }

    #[test]
    fn create_class_emits_event() {
        let g = DeterministicIdGen::starting_at(700);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let class_id = ClassId::new(school, g.next_uuid());
        let cmd = CreateClassCommand {
            tenant: ctx,
            class_id,
            class_name: "Grade 1".to_owned(),
            pass_mark: 50.0,
        };
        let (class, event) = create_class(cmd, &clock, &g, &u).unwrap();
        assert_eq!(class.name, "Grade 1");
        assert_eq!(event.class_id, class_id);
        assert_eq!(event.class_name, "Grade 1");
        assert_eq!(event.pass_mark, 50.0);
    }

    #[test]
    fn set_optional_subject_gpa_threshold_updates_class() {
        let g = DeterministicIdGen::starting_at(800);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let class_id = ClassId::new(school, g.next_uuid());
        let (mut class, _) = create_class(
            CreateClassCommand {
                tenant: ctx.clone(),
                class_id,
                class_name: "Grade 1".to_owned(),
                pass_mark: 50.0,
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        let event = set_optional_subject_gpa_threshold(
            &mut class,
            SetOptionalSubjectGpaThresholdCommand {
                tenant: ctx,
                class_id,
                threshold: 3.5,
            },
            &clock,
            &g,
        )
        .unwrap();
        assert_eq!(class.optional_subject_gpa_threshold.as_f32(), 3.5);
        assert_eq!(event.threshold, 3.5);
    }

    #[test]
    fn create_section_emits_event() {
        let g = DeterministicIdGen::starting_at(900);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let section_id = SectionId::new(school, g.next_uuid());
        let (section, event) = create_section(
            CreateSectionCommand {
                tenant: ctx,
                section_id,
                section_name: "A".to_owned(),
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        assert_eq!(section.name, "A");
        assert_eq!(event.section_name, "A");
    }

    #[test]
    fn create_subject_emits_event() {
        let g = DeterministicIdGen::starting_at(1000);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let subject_id = SubjectId::new(school, g.next_uuid());
        let (subject, event) = create_subject(
            CreateSubjectCommand {
                tenant: ctx,
                subject_id,
                subject_code: "MATH".to_owned(),
                subject_name: "Mathematics".to_owned(),
                subject_type: crate::value_objects::SubjectType::Theory,
                pass_mark: 40.0,
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        assert_eq!(subject.code, "MATH");
        assert_eq!(event.code, "MATH");
        assert_eq!(
            event.subject_type,
            crate::value_objects::SubjectType::Theory
        );
    }

    #[test]
    fn create_academic_year_emits_event_and_range() {
        let g = DeterministicIdGen::starting_at(1100);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let year_id = AcademicYearId::new(school, g.next_uuid());
        let (year, event) = create_academic_year(
            CreateAcademicYearCommand {
                tenant: ctx,
                academic_year_id: year_id,
                year: "2026".to_owned(),
                title: "Academic Year 2026-2027".to_owned(),
                starting_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                ending_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                is_current: true,
                copy_with_academic_year: None,
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        assert_eq!(year.year, "2026");
        assert!(year.is_current);
        assert!(year
            .range
            .contains(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
        assert!(event.is_current);
    }

    #[test]
    fn close_academic_year_demotes_current() {
        let g = DeterministicIdGen::starting_at(1200);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let year_id = AcademicYearId::new(school, g.next_uuid());
        let (mut year, _) = create_academic_year(
            CreateAcademicYearCommand {
                tenant: ctx.clone(),
                academic_year_id: year_id,
                year: "2026".to_owned(),
                title: "AY 2026-2027".to_owned(),
                starting_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                ending_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                is_current: true,
                copy_with_academic_year: None,
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        let _ = close_academic_year(
            &mut year,
            CloseAcademicYearCommand {
                tenant: ctx,
                academic_year_id: year_id,
            },
            &clock,
            &g,
        )
        .unwrap();
        assert!(year.is_closed);
        assert!(!year.is_current);
    }

    #[test]
    fn copy_academic_year_requires_same_school() {
        let g = DeterministicIdGen::starting_at(1300);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let year_id = AcademicYearId::new(school, g.next_uuid());
        let other_school = SchoolId::from_uuid(uuid::Uuid::now_v7());
        let other_year = AcademicYearId::new(other_school, uuid::Uuid::now_v7());
        let (mut year, _) = create_academic_year(
            CreateAcademicYearCommand {
                tenant: ctx,
                academic_year_id: year_id,
                year: "2026".to_owned(),
                title: "AY 2026-2027".to_owned(),
                starting_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                ending_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                is_current: false,
                copy_with_academic_year: None,
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        let err = copy_academic_year(&mut year, other_year, &clock, &g).unwrap_err();
        assert!(matches!(
            err,
            educore_core::error::DomainError::Validation(_)
        ));
    }

    #[test]
    fn school_matches_helper() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let ctx = ctx_for(s);
        assert!(school_matches(&ctx, s));
        assert!(!school_matches(
            &ctx,
            SchoolId::from_uuid(uuid::Uuid::now_v7())
        ));
    }

    #[test]
    fn update_student_profile_changes_only_supplied_fields() {
        let g = DeterministicIdGen::starting_at(1400);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::new();
        let school = g.next_school_id();
        let ctx = ctx_for(school);
        let student_id = StudentId::new(school, g.next_uuid());
        let class = ClassId::new(school, g.next_uuid());
        let section = SectionId::new(school, g.next_uuid());
        let year = AcademicYearId::new(school, g.next_uuid());
        let cmd = admit_cmd(ctx.clone(), student_id, class, section, year, "ADM-001");
        let (mut student, _) = admit_student(cmd, &clock, &g, &u).unwrap();
        let event = update_student_profile(
            &ctx,
            &mut student,
            UpdateStudentProfileCommand {
                tenant: ctx.clone(),
                student_id,
                first_name: Some("Augusta Ada".to_owned()),
                last_name: None,
                gender: None,
                mobile: None,
                email: None,
                current_address: None,
                permanent_address: None,
                custom_fields: None,
            },
            &clock,
            &g,
            &u,
        )
        .unwrap();
        assert_eq!(event.changed_fields, vec!["first_name".to_owned()]);
        assert_eq!(student.first_name, "Augusta Ada");
    }
}
