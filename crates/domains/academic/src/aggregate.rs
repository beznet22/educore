//! # Academic aggregate roots
//!
//! The five prompt-named aggregates Phase 3 ships:
//!
//! - [`Student`] — a person enrolled in a school. The
//!   canonical Phase 3 deliverable.
//! - [`Class`] — a grade level offered by the school
//!   (e.g. "Grade 1", "Year 7").
//! - [`Section`] — a division of a class (e.g. "Section A").
//! - [`Subject`] — a course subject (e.g. "Mathematics").
//! - [`AcademicYear`] — a school year with a date range.
//!
//! All five follow the "aggregate as a single struct" pattern
//! (mirroring `educore-platform`'s [`School`](educore_platform::School)
//! and [`User`](educore_platform::User)): the struct holds the
//! full state, with `version` for optimistic concurrency,
//! `etag` for content hashing, `active_status` for soft
//! delete, and `last_event_id` / `correlation_id` for the
//! audit / outbox bridge.
//!
//! Each aggregate is **tenant-scoped** via its typed id
//! (e.g. `StudentId(SchoolId, Uuid)`) so the type system
//! catches cross-tenant confusion at compile time.

use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    AcademicYearId, AcademicYearRange, CertificateId, ClassId, ClassPeriod, ClassRoutineId,
    ClassRoomId, ClassSectionId, ClassSubjectId, ClassSubjectScope, DayOfWeek, EmailAddress,
    GuardianId, HomeworkId, IdCardId, LessonId, LessonPlanId, LessonTopicId,
    OptionalSubjectAssignmentId, OptionalSubjectGpaThreshold, PassMark, PhoneNumber,
    RegistrationFieldId, Relation, SectionId, StudentCategoryId, StudentGroupId,
    StudentGuardianLinkId, StudentId, StudentPromotionId, StudentRecordId, SubjectId, SubjectType,
};

/// Returns the default etag for a freshly minted aggregate.
///
/// Delegates to [`Etag::placeholder`] (an infallible
/// constructor) so callers do not need to handle a `Result`.
fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// Student
// =============================================================================

/// A person enrolled in a school.
///
/// The aggregate owns the student's identity, profile,
/// status, and audit metadata. Children (`StudentRecord`,
/// `StudentCategory` membership, `StudentGroup` membership,
/// `OptionalSubjectAssignment`, `StudentDocument`,
/// `StudentTimeline`, `StudentHomework`) are tracked
/// separately; per the prompt, only the `Student` aggregate
/// is in Phase 3 scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Student {
    /// The student's typed id.
    pub id: StudentId,
    /// The school the student belongs to (tenant anchor; also
    /// embedded in the typed id).
    pub school_id: SchoolId,
    /// The student's admission number (unique within school).
    pub admission_no: String,
    /// The student's first name.
    pub first_name: String,
    /// The student's last name.
    pub last_name: String,
    /// The student's date of birth.
    pub date_of_birth: chrono::NaiveDate,
    /// The student's gender.
    pub gender: crate::value_objects::Gender,
    /// The student's blood group (optional).
    pub blood_group: Option<crate::value_objects::BloodGroup>,
    /// The student's religion (free-form, optional).
    pub religion: Option<String>,
    /// The student's caste (free-form, optional).
    pub caste: Option<String>,
    /// The student's mobile phone number (E.164, optional).
    pub mobile: Option<String>,
    /// The student's email (validated, optional).
    pub email: Option<String>,
    /// The student's current address (optional).
    pub current_address: Option<String>,
    /// The student's permanent address (optional).
    pub permanent_address: Option<String>,
    /// The date the student was admitted.
    pub admission_date: chrono::NaiveDate,
    /// The class the student is admitted into.
    pub class_id: ClassId,
    /// The section the student is admitted into.
    pub section_id: SectionId,
    /// The academic year the admission applies to.
    pub academic_year_id: AcademicYearId,
    /// The student's roll number (optional; can be set later).
    pub roll_no: Option<String>,
    /// The student's current lifecycle status.
    pub status: crate::value_objects::StudentStatus,
    /// Custom fields (e.g. parent-supplied free-text data).
    pub custom_fields: std::collections::BTreeMap<String, String>,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash for conditional writes.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// User that created the student.
    pub created_by: UserId,
    /// User that last mutated the student.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id that touched this student.
    pub last_event_id: Option<EventId>,
    /// Correlation id of the request that created this student.
    pub correlation_id: CorrelationId,
}

impl Student {
    /// The default etag for a freshly minted student.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `Student` in its just-minted state.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: StudentId,
        admission_no: String,
        first_name: String,
        last_name: String,
        date_of_birth: chrono::NaiveDate,
        gender: crate::value_objects::Gender,
        admission_date: chrono::NaiveDate,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        roll_no: Option<String>,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id: id.school_id(),
            admission_no,
            first_name,
            last_name,
            date_of_birth,
            gender,
            blood_group: None,
            religion: None,
            caste: None,
            mobile: None,
            email: None,
            current_address: None,
            permanent_address: None,
            admission_date,
            class_id,
            section_id,
            academic_year_id,
            roll_no,
            status: crate::value_objects::StudentStatus::Active,
            custom_fields: std::collections::BTreeMap::new(),
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns the student's computed full name.
    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

// =============================================================================
// Class
// =============================================================================

/// A grade level offered by the school (e.g. "Grade 1").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    /// The class's typed id.
    pub id: ClassId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The class's display name (unique within school).
    pub name: String,
    /// The class's pass mark (0.0..=100.0).
    pub pass_mark: PassMark,
    /// The optional-subject GPA threshold (0.0..=5.0). The
    /// default threshold is `0.0` (any GPA is eligible); the
    /// school can raise it via `SetOptionalSubjectGpaThreshold`.
    pub optional_subject_gpa_threshold: OptionalSubjectGpaThreshold,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl Class {
    /// The default etag for a freshly minted class.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `Class` in its just-minted state.
    #[must_use]
    pub fn fresh(
        id: ClassId,
        name: String,
        pass_mark: PassMark,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id: id.school_id(),
            name,
            pass_mark,
            optional_subject_gpa_threshold: OptionalSubjectGpaThreshold::new(0.0).unwrap_or_else(
                |_| unreachable!("0.0 is in the valid OptionalSubjectGpaThreshold range"),
            ),
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// Section
// =============================================================================

/// A division of a class (e.g. "Section A").
///
/// Sections can be reused across multiple academic years;
/// the per-year pairing is the `ClassSection` aggregate (a
/// later phase).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    /// The section's typed id.
    pub id: SectionId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The section's display name (unique within school).
    pub name: String,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl Section {
    /// The default etag for a freshly minted section.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `Section` in its just-minted state.
    #[must_use]
    pub fn fresh(
        id: SectionId,
        name: String,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id: id.school_id(),
            name,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// Subject
// =============================================================================

/// A subject offered in a class (e.g. "Mathematics").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    /// The subject's typed id.
    pub id: SubjectId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The subject's code (unique within school).
    pub code: String,
    /// The subject's display name.
    pub name: String,
    /// The subject's type (Theory or Practical).
    pub subject_type: SubjectType,
    /// The subject's pass mark (0.0..=100.0).
    pub pass_mark: PassMark,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl Subject {
    /// The default etag for a freshly minted subject.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `Subject` in its just-minted state.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: SubjectId,
        code: String,
        name: String,
        subject_type: SubjectType,
        pass_mark: PassMark,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id: id.school_id(),
            code,
            name,
            subject_type,
            pass_mark,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// AcademicYear
// =============================================================================

/// A school year with a defined start and end date.
///
/// Per the spec, exactly one academic year may be marked
/// `current` per school at a time. The `SetCurrentAcademicYear`
/// command is the only mutator of the `is_current` flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYear {
    /// The academic year's typed id.
    pub id: AcademicYearId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The academic year's short label (e.g. "2026").
    pub year: String,
    /// The academic year's display title.
    pub title: String,
    /// The academic year's date range.
    pub range: AcademicYearRange,
    /// Whether this is the current academic year for the
    /// school. Exactly one row per school may have
    /// `is_current = true`.
    pub is_current: bool,
    /// Whether the academic year is closed (read-only).
    pub is_closed: bool,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl AcademicYear {
    /// The default etag for a freshly minted academic year.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns an `AcademicYear` in its just-minted state.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: AcademicYearId,
        year: String,
        title: String,
        range: AcademicYearRange,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id: id.school_id(),
            year,
            title,
            range,
            is_current: false,
            is_closed: false,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// Placeholder aggregates for the remaining 14 academic aggregates.
//
// Minimal `id + school_id` stubs so the spec is exhaustively
// representable in code. The full impl (audit footer, domain
// fields, invariants, services, events) lands in subsequent
// workstreams per `docs/build-plan.md`.
//
// Each stub uses the typed id from `crate::value_objects` so the
// type system catches cross-tenant confusion at compile time
// (the `school_id` is derived from `id.school_id()` in real impl;
// it is held explicitly here so the placeholder round-trips
// through Serde without depending on a `Default::default` for
// the id).
// =============================================================================

macro_rules! academic_aggregate_stub {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            id: $id_ty:ty $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        $vis struct $name {
            /// The typed id (school_id + uuid).
            pub id: $id_ty,
            /// The owning school (derived from `id.school_id()` in
            /// real impl; held explicitly here so the placeholder
            /// is self-contained).
            pub school_id: SchoolId,
        }
    };
}

// =============================================================================
// Guardian (Cluster D, Batch 1: full impl)
// =============================================================================

/// A parent, legal guardian, or authorized contact for a
/// [`Student`].
///
/// Per `docs/specs/academic/aggregates.md` § Guardian:
///
/// - I-1: At most one phone and one email of record.
/// - I-2: A guardian may be linked to multiple students.
/// - I-5: A guardian is soft-deleted when all their student
///   links are removed.
///
/// The aggregate's contact fields are
/// [`PhoneNumber`]/[`EmailAddress`] (validated at construction).
/// The first/last name are stored as `String` to match the
/// engine's permissive-on-strings profile field pattern
/// (the value object pattern is reserved for ids and
/// validated inputs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guardian {
    /// The guardian's typed id.
    pub id: GuardianId,
    /// The owning school (tenant anchor; also embedded in the
    /// typed id).
    pub school_id: SchoolId,
    /// The guardian's first name.
    pub first_name: String,
    /// The guardian's last name.
    pub last_name: String,
    /// The guardian's phone number of record (optional). Per
    /// I-1, at most one phone is carried per guardian.
    pub phone: Option<PhoneNumber>,
    /// The guardian's email of record (optional). Per I-1,
    /// at most one email is carried per guardian.
    pub email: Option<EmailAddress>,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag. Per I-5, the guardian is set to
    /// `Retired` when the last student link is removed.
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl Guardian {
    /// The default etag for a freshly minted guardian.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `Guardian` in its just-minted state.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: GuardianId,
        first_name: String,
        last_name: String,
        phone: Option<PhoneNumber>,
        email: Option<EmailAddress>,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        Self {
            id,
            school_id: id.school_id(),
            first_name,
            last_name,
            phone,
            email,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Returns the guardian's computed full name.
    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

// =============================================================================
// StudentGuardianLink (Cluster D, Batch 1: full impl)
// =============================================================================

/// The linkage between a [`Guardian`] and a [`Student`],
/// carrying the [`Relation`] and the `IsPrimary` flag.
///
/// Per `docs/specs/academic/aggregates.md` § Guardian:
///
/// - I-2: Multi-student linkage (a guardian may be linked to
///   multiple students).
/// - I-3: A link carries `Relation` + `IsPrimary`.
/// - I-4: At most one `IsPrimary` per student.
/// - I-5: When the last link is removed, the guardian is
///   soft-deleted (the link is deleted; the guardian's
///   `active_status` is flipped to `Retired` by the
///   service).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentGuardianLink {
    /// The link's typed id.
    pub id: StudentGuardianLinkId,
    /// The owning school (tenant anchor).
    pub school_id: SchoolId,
    /// The guardian being linked.
    pub guardian_id: GuardianId,
    /// The student being linked.
    pub student_id: StudentId,
    /// The relationship.
    pub relation: Relation,
    /// Whether this guardian is the primary contact for the
    /// student (used for communication routing).
    pub is_primary: bool,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag. Links are hard-deleted when the
    /// relationship is severed (the link itself is the unit
    /// of work; the guardian is soft-deleted only when the
    /// last link is removed per I-5).
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl StudentGuardianLink {
    /// The default etag for a freshly minted link.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `StudentGuardianLink` in its just-minted state.
    #[must_use]
    pub fn fresh(
        id: StudentGuardianLinkId,
        guardian_id: GuardianId,
        student_id: StudentId,
        relation: Relation,
        is_primary: bool,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        // Enforce I-2 invariant at the type-system level:
        // the link must belong to the same school as both
        // the guardian and the student. The engine rejects
        // cross-tenant ids at the service boundary; we
        // assert here as a last-line defense for in-process
        // callers.
        debug_assert_eq!(guardian_id.school_id(), student_id.school_id());
        debug_assert_eq!(guardian_id.school_id(), id.school_id());
        Self {
            id,
            school_id: id.school_id(),
            guardian_id,
            student_id,
            relation,
            is_primary,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// OptionalSubjectAssignment (Cluster D, Batch 1: full impl)
// =============================================================================

/// The assignment of an optional subject to a student for a
/// specific academic year.
///
/// Per `docs/specs/academic/aggregates.md` § Student § I-4:
/// a student can be in at most one optional subject per
/// academic year. The aggregate is its own root so the
/// storage adapter can enforce the constraint via a unique
/// index on `(school_id, student_id, academic_year_id)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalSubjectAssignment {
    /// The assignment's typed id.
    pub id: OptionalSubjectAssignmentId,
    /// The owning school (tenant anchor).
    pub school_id: SchoolId,
    /// The student receiving the optional subject.
    pub student_id: StudentId,
    /// The optional subject being assigned.
    pub subject_id: SubjectId,
    /// The academic year the assignment applies to.
    pub academic_year_id: AcademicYearId,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id.
    pub last_event_id: Option<EventId>,
    /// Correlation id.
    pub correlation_id: CorrelationId,
}

impl OptionalSubjectAssignment {
    /// The default etag for a freshly minted assignment.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns an `OptionalSubjectAssignment` in its
    /// just-minted state.
    #[must_use]
    pub fn fresh(
        id: OptionalSubjectAssignmentId,
        student_id: StudentId,
        subject_id: SubjectId,
        academic_year_id: AcademicYearId,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let etag = fresh_etag();
        // Cross-tenant guard: all three ids must share a
        // school. The service layer enforces the same
        // invariant before calling `fresh`.
        debug_assert_eq!(student_id.school_id(), id.school_id());
        debug_assert_eq!(subject_id.school_id(), id.school_id());
        debug_assert_eq!(academic_year_id.school_id(), id.school_id());
        Self {
            id,
            school_id: id.school_id(),
            student_id,
            subject_id,
            academic_year_id,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}
academic_aggregate_stub! {
    /// The pairing of a class and a section in a specific
    /// academic year that students are enrolled into. See
    /// `docs/specs/academic/aggregates.md` § ClassSection.
    pub struct _ClassSectionStub { id: ClassSectionId }
}

// =============================================================================
// ClassSection (Cluster D, Batch 2: full impl)
// =============================================================================

/// A pairing of a class and a section in a specific academic
/// year that students are enrolled into and that class
/// routines are scheduled against.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSection:
///
/// - I-1: Unique per `(class, section, academic_year)`.
///   Enforced by the service via
///   `UniquenessChecker::class_section_exists`.
/// - I-3: A `ClassSection` has one or more `ClassRoomId`s
///   assigned. Enforced by `ClassSection::fresh` rejecting
///   empty `class_rooms`.
/// - I-4: A `ClassSection` cannot be deleted while
///   `StudentRecord`s reference it. Enforced by the service
///   via `UniquenessChecker::class_section_has_student_records`.
///
/// I-2 (multiple class / subject teachers) is permissive
/// and does not require a constraint in the aggregate
/// (the aggregate tracks teacher ids via separate events;
/// any number of teacher ids is data-permitted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassSection {
    /// The class-section's typed id.
    pub id: ClassSectionId,
    /// The owning school (tenant anchor; also embedded in
    /// the typed id).
    pub school_id: SchoolId,
    /// The class this section is a division of.
    pub class_id: ClassId,
    /// The section within the class.
    pub section_id: SectionId,
    /// The academic year this pairing applies to.
    pub academic_year_id: AcademicYearId,
    /// The class rooms assigned to this section. Per I-3,
    /// must contain at least one entry.
    pub class_rooms: Vec<ClassRoomId>,
    /// Whether the section is currently active (true) or
    /// soft-deleted / retired (false). Set to `false` by
    /// `ClassSection::retire`.
    pub is_active: bool,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag. Set to `Retired` when
    /// `delete_class_section` succeeds (only permitted when
    /// no `StudentRecord`s reference this class-section).
    pub active_status: ActiveStatus,
    /// Last event id (for the outbox / audit bridge).
    pub last_event_id: Option<EventId>,
    /// Correlation id of the request that originated this
    /// aggregate.
    pub correlation_id: CorrelationId,
}

impl ClassSection {
    /// The default etag for a freshly minted class-section.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `ClassSection` in its just-minted state.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if `class_rooms` is
    /// empty (I-3) or if the typed id's school does not match
    /// `class_id.school_id()`, `section_id.school_id()`, or
    /// `academic_year_id.school_id()`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn fresh(
        id: ClassSectionId,
        class_id: ClassId,
        section_id: SectionId,
        academic_year_id: AcademicYearId,
        class_rooms: Vec<ClassRoomId>,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> std::result::Result<Self, educore_core::error::DomainError> {
        use educore_core::error::DomainError;
        // Tenant-anchor invariant: every referenced id must
        // share the same school as the typed class-section id.
        if class_id.school_id() != id.school_id() {
            return Err(DomainError::Validation(format!(
                "class_id {class_id} is in school {}, class_section id school is {}",
                class_id.school_id(),
                id.school_id()
            )));
        }
        if section_id.school_id() != id.school_id() {
            return Err(DomainError::Validation(format!(
                "section_id {section_id} is in school {}, class_section id school is {}",
                section_id.school_id(),
                id.school_id()
            )));
        }
        if academic_year_id.school_id() != id.school_id() {
            return Err(DomainError::Validation(format!(
                "academic_year_id {academic_year_id} is in school {}, class_section id school is {}",
                academic_year_id.school_id(),
                id.school_id()
            )));
        }
        // I-3: one or more class rooms.
        if class_rooms.is_empty() {
            return Err(DomainError::Validation(
                "class_rooms must contain at least one ClassRoomId (spec I-3)".to_owned(),
            ));
        }
        let etag = fresh_etag();
        Ok(Self {
            id,
            school_id: id.school_id(),
            class_id,
            section_id,
            academic_year_id,
            class_rooms,
            is_active: true,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Append a class room to the section's `class_rooms`
    /// list. Caller is responsible for uniqueness / business
    /// rule checks (the service layer enforces them).
    ///
    /// Bumps `updated_at`, `updated_by`, `version`, and
    /// `last_event_id`.
    pub fn add_class_room(
        &mut self,
        class_room: ClassRoomId,
        updated_by: UserId,
        now: Timestamp,
        event_id: EventId,
    ) {
        self.class_rooms.push(class_room);
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-delete the class-section. Sets
    /// `active_status = Retired` and `is_active = false`,
    /// bumps the audit footer. The service is responsible
    /// for verifying that no `StudentRecord`s reference
    /// this class-section (I-4) before calling this method.
    pub fn retire(&mut self, updated_by: UserId, now: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.is_active = false;
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}
// =============================================================================
// ClassSubject
// =============================================================================

/// The assignment of a subject to a class (and possibly a
/// specific section), with a teacher and an optional
/// `PassMark` override. See
/// `docs/specs/academic/aggregates.md` § ClassSubject.
///
/// Per the spec:
///
/// - **I-1** (Class or class-section scope): the
///   `scope` field is a closed [`ClassSubjectScope`] enum
///   (`ClassOnly` | `ClassSection`). `ClassOnly` requires
///   `class_section_id == None`; `ClassSection` requires
///   `class_section_id == Some(_)`. Enforced by
///   [`ClassSubject::fresh`].
/// - **I-2** (Same teacher may be assigned to multiple
///   class-subjects) is permissive; the data model
///   permits any number of class-subjects per teacher.
/// - **I-3** (`PassMark` override): the `pass_mark` field
///   is an `Option<PassMark>`. When `Some`, the inner
///   [`PassMark`] value object validates the range
///   `0.0..=100.0`. Enforced by [`ClassSubject::fresh`]
///   (which constructs `PassMark` via the value object
///   constructor that rejects out-of-range values).
///
/// I-2 is permissive; no constraint is needed in the
/// aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassSubject {
    /// The class-subject's typed id.
    pub id: ClassSubjectId,
    /// The owning school (tenant anchor; also embedded in
    /// the typed id).
    pub school_id: SchoolId,
    /// The class this subject is assigned to.
    pub class_id: ClassId,
    /// The class-section this subject is assigned to.
    /// `Some` when `scope == ClassSection`, `None` when
    /// `scope == ClassOnly`. Enforced by [`Self::fresh`]
    /// per I-1.
    pub class_section_id: Option<ClassSectionId>,
    /// The subject being assigned.
    pub subject_id: SubjectId,
    /// The teacher assigned to teach this class-subject.
    /// Typed as [`UserId`] (no `StaffId` exists in the
    /// academic crate today; teachers are users).
    pub teacher_id: UserId,
    /// The scope of the assignment (`ClassOnly` or
    /// `ClassSection`). Cross-field validated with
    /// `class_section_id` per I-1.
    pub scope: ClassSubjectScope,
    /// Optional per-class-subject `PassMark` override. When
    /// `Some`, the inner value must be in `0.0..=100.0`
    /// per I-3 (enforced by [`PassMark::new`]).
    pub pass_mark: Option<PassMark>,
    /// Whether this class-subject is currently active.
    /// Set to `false` by [`ClassSubject::retire`].
    pub is_active: bool,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag. Set to `Retired` by
    /// [`ClassSubject::retire`].
    pub active_status: ActiveStatus,
    /// Last event id (for the outbox / audit bridge).
    pub last_event_id: Option<EventId>,
    /// Correlation id of the request that originated this
    /// aggregate.
    pub correlation_id: CorrelationId,
}

impl ClassSubject {
    /// The default etag for a freshly minted class-subject.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `ClassSubject` in its just-minted state.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if:
    /// - `class_id.school_id() != id.school_id()`,
    ///   `class_section_id.school_id() != id.school_id()`
    ///   (when `Some`), or
    ///   `subject_id.school_id() != id.school_id()`
    ///   (tenant-anchor invariant);
    /// - `scope == ClassOnly` but `class_section_id` is
    ///   `Some` (I-1 violation);
    /// - `scope == ClassSection` but `class_section_id` is
    ///   `None` (I-1 violation);
    /// - `pass_mark.is_some_and` the inner value is outside
    ///   `0.0..=100.0` (I-3 violation, via [`PassMark::new`]).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ClassSubjectId,
        class_id: ClassId,
        class_section_id: Option<ClassSectionId>,
        subject_id: SubjectId,
        teacher_id: UserId,
        scope: ClassSubjectScope,
        pass_mark: Option<PassMark>,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        use educore_core::error::DomainError;
        // Tenant-anchor invariant: every referenced id must
        // share the same school as the typed class-subject id.
        if class_id.school_id() != id.school_id() {
            return Err(DomainError::Validation(format!(
                "class_id {class_id} is in school {}, class_subject id school is {}",
                class_id.school_id(),
                id.school_id()
            )));
        }
        if subject_id.school_id() != id.school_id() {
            return Err(DomainError::Validation(format!(
                "subject_id {subject_id} is in school {}, class_subject id school is {}",
                subject_id.school_id(),
                id.school_id()
            )));
        }
        if let Some(cs) = class_section_id {
            if cs.school_id() != id.school_id() {
                return Err(DomainError::Validation(format!(
                    "class_section_id {cs} is in school {}, class_subject id school is {}",
                    cs.school_id(),
                    id.school_id()
                )));
            }
        }
        // I-1: Class or class-section scope cross-field
        // consistency.
        match scope {
            ClassSubjectScope::ClassOnly => {
                if class_section_id.is_some() {
                    return Err(DomainError::Validation(format!(
                        "ClassOnly scope cannot have class_section_id set \
                         (got class_section_id {class_section_id:?}, class_subject {id})"
                    )));
                }
            }
            ClassSubjectScope::ClassSection => {
                if class_section_id.is_none() {
                    return Err(DomainError::Validation(format!(
                        "ClassSection scope requires class_section_id to be Some \
                         (got None for class_subject {id})"
                    )));
                }
            }
        }
        // I-3: PassMark override range check. The constructor
        // already rejects out-of-range values, but we surface
        // the error here so callers see the precise violation.
        if let Some(pm_value) = pass_mark {
            // Reconstruct via `PassMark::new` to assert the
            // invariant end-to-end (the option could carry a
            // value constructed via `PassMark::new`, but we
            // validate again to guarantee any future
            // caller-construction paths are still checked).
            let _ = PassMark::new(pm_value.as_f32())?;
        }
        let etag = fresh_etag();
        Ok(Self {
            id,
            school_id: id.school_id(),
            class_id,
            class_section_id,
            subject_id,
            teacher_id,
            scope,
            pass_mark,
            is_active: true,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Replace the current teacher with `new_teacher_id`.
    /// Bumps `updated_at`, `updated_by`, `version`, and
    /// `last_event_id`. Returns the previous `teacher_id`
    /// for the event payload.
    pub fn reassign_teacher(
        &mut self,
        new_teacher_id: UserId,
        updated_by: UserId,
        now: Timestamp,
        event_id: EventId,
    ) -> UserId {
        let previous = self.teacher_id;
        self.teacher_id = new_teacher_id;
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        previous
    }

    /// Soft-delete (retire) the class-subject. Sets
    /// `active_status = Retired` and `is_active = false`,
    /// bumps the audit footer. The service layer is
    /// responsible for any business preconditions (per
    /// the spec, `unassign_subject` is unconditional —
    /// any active class-subject may be unassigned).
    pub fn retire(&mut self, updated_by: UserId, now: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.is_active = false;
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}
// =============================================================================
// ClassRoutine (Cluster D, Batch 2: full impl — Wave 51)
// =============================================================================

/// The weekly schedule for a [`ClassSection`].
///
/// Per `docs/specs/academic/aggregates.md` § ClassRoutine:
///
/// - **I-1**: a routine covers a full week (7 distinct
///   days Mon-Sun). Enforced by [`Self::fresh`] and
///   [`Self::replace_periods`].
/// - **I-2**: periods are identified by `ClassTimeId`;
///   no two periods in the same routine may share a
///   `class_time_id`. Enforced by [`Self::fresh`] and
///   [`Self::replace_periods`].
/// - **I-3**: every period carries both a `room_id`
///   and a `teacher_id` (compile-time enforced; the
///   `ClassPeriod` struct has no `None` slots).
/// - **I-4** / **I-5**: teacher / room no-conflict are
///   enforced at the **service boundary** via
///   [`crate::commands::UniquenessChecker::teacher_has_conflict`]
///   and [`crate::commands::UniquenessChecker::room_has_conflict`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassRoutine {
    /// The class-routine's typed id.
    pub id: ClassRoutineId,
    /// The owning school (tenant anchor; also embedded in
    /// the typed id).
    pub school_id: SchoolId,
    /// The class-section this routine is scheduled for.
    pub class_section_id: ClassSectionId,
    /// The academic year this routine applies to.
    pub academic_year_id: AcademicYearId,
    /// The full-week period schedule. Per I-1, must
    /// cover all 7 distinct days; per I-2, no two periods
    /// may share a `class_time_id`.
    pub periods: Vec<ClassPeriod>,
    /// Whether this routine is currently active. Set to
    /// `false` by [`Self::retire`].
    pub is_active: bool,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// Created by.
    pub created_by: UserId,
    /// Last mutated by.
    pub updated_by: UserId,
    /// Soft-delete flag. Set to `Retired` by
    /// [`Self::retire`].
    pub active_status: ActiveStatus,
    /// Last event id (for the outbox / audit bridge).
    pub last_event_id: Option<EventId>,
    /// Correlation id of the request that originated this
    /// aggregate.
    pub correlation_id: CorrelationId,
}

impl ClassRoutine {
    /// The default etag for a freshly minted class-routine.
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Returns a `ClassRoutine` in its just-minted state.
    ///
    /// Per `docs/specs/academic/aggregates.md` § ClassRoutine:
    ///
    /// - **I-1**: rejects if `periods` does not cover all 7
    ///   distinct days (Mon-Sun).
    /// - **I-2**: rejects if two periods share the same
    ///   `class_time_id`.
    /// - **I-3**: every period must validate via
    ///   [`ClassPeriod::validate`] (per-period
    ///   structural check: `period_number >= 1`).
    ///
    /// The I-4 (teacher no-conflict) and I-5 (room
    /// no-conflict) invariants are **not** enforced here —
    /// they are checked at the service boundary via
    /// [`crate::commands::UniquenessChecker`], since the
    /// conflict check requires storage-layer state
    /// (existing routines in the same school).
    ///
    /// # Errors
    ///
    /// - `DomainError::Validation` if any period has
    ///   `period_number == 0` (per-period I-3 structural
    ///   check), or if the period set does not cover all 7
    ///   distinct days (I-1).
    /// - `DomainError::Conflict` if two periods share the
    ///   same `class_time_id` (I-2).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ClassRoutineId,
        class_section_id: ClassSectionId,
        academic_year_id: AcademicYearId,
        periods: Vec<ClassPeriod>,
        created_by: UserId,
        updated_by: UserId,
        now: Timestamp,
        correlation_id: CorrelationId,
    ) -> educore_core::error::Result<Self> {
        use educore_core::error::DomainError;
        // Tenant-anchor invariant: every referenced id must
        // share the same school as the typed class-routine
        // id.
        if class_section_id.school_id() != id.school_id() {
            return Err(DomainError::validation(format!(
                "class_section_id {class_section_id} is in school {}, \
                 class_routine id school is {}",
                class_section_id.school_id(),
                id.school_id()
            )));
        }
        if academic_year_id.school_id() != id.school_id() {
            return Err(DomainError::validation(format!(
                "academic_year_id {academic_year_id} is in school {}, \
                 class_routine id school is {}",
                academic_year_id.school_id(),
                id.school_id()
            )));
        }
        // I-3 (per-period structural): each period must
        // pass `ClassPeriod::validate`.
        for (idx, p) in periods.iter().enumerate() {
            p.validate().map_err(|e| {
                DomainError::validation(format!("period #{idx}: {e}"))
            })?;
        }
        // I-1: full week — 7 distinct days.
        let mut days: HashSet<DayOfWeek> = HashSet::with_capacity(7);
        for p in &periods {
            days.insert(p.day);
        }
        if days.len() != 7 {
            return Err(DomainError::validation(format!(
                "ClassRoutine must cover all 7 days, got {} distinct days",
                days.len()
            )));
        }
        // I-2: no duplicate ClassTimeId.
        let mut class_times: HashSet<crate::value_objects::ClassTimeId> =
            HashSet::with_capacity(periods.len());
        for p in &periods {
            if !class_times.insert(p.class_time_id) {
                return Err(DomainError::conflict(format!(
                    "duplicate class_time_id: {:?}",
                    p.class_time_id
                )));
            }
        }
        let etag = fresh_etag();
        Ok(Self {
            id,
            school_id: id.school_id(),
            class_section_id,
            academic_year_id,
            periods,
            is_active: true,
            version: Version::initial(),
            etag,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Replace the current period set with `new_periods`.
    ///
    /// Per `docs/specs/academic/aggregates.md` § ClassRoutine,
    /// the same I-1 / I-2 / I-3 invariants enforced by
    /// [`Self::fresh`] are re-checked. Bumps
    /// `updated_at`, `updated_by`, `version`, and
    /// `last_event_id`. Returns the previous period set
    /// for the event payload.
    ///
    /// # Errors
    ///
    /// - `DomainError::Validation` if any period has
    ///   `period_number == 0` (I-3 per-period), or if the
    ///   new period set does not cover all 7 distinct days
    ///   (I-1).
    /// - `DomainError::Conflict` if two new periods share
    ///   the same `class_time_id` (I-2).
    pub fn replace_periods(
        &mut self,
        new_periods: Vec<ClassPeriod>,
        updated_by: UserId,
        now: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<Vec<ClassPeriod>> {
        use educore_core::error::DomainError;
        for (idx, p) in new_periods.iter().enumerate() {
            p.validate().map_err(|e| {
                DomainError::validation(format!("period #{idx}: {e}"))
            })?;
        }
        let mut days: HashSet<DayOfWeek> = HashSet::with_capacity(7);
        for p in &new_periods {
            days.insert(p.day);
        }
        if days.len() != 7 {
            return Err(DomainError::validation(format!(
                "ClassRoutine must cover all 7 days, got {} distinct days",
                days.len()
            )));
        }
        let mut class_times: HashSet<crate::value_objects::ClassTimeId> =
            HashSet::with_capacity(new_periods.len());
        for p in &new_periods {
            if !class_times.insert(p.class_time_id) {
                return Err(DomainError::conflict(format!(
                    "duplicate class_time_id: {:?}",
                    p.class_time_id
                )));
            }
        }
        let previous = std::mem::replace(&mut self.periods, new_periods);
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(previous)
    }

    /// Swap two periods by index in the current `periods`
    /// vector. Returns the previous `(a, b)` pair for the
    /// event payload.
    ///
    /// # Errors
    ///
    /// - `DomainError::Validation` if either index is
    ///   out of bounds.
    pub fn swap_periods(
        &mut self,
        period_a_idx: usize,
        period_b_idx: usize,
        updated_by: UserId,
        now: Timestamp,
        event_id: EventId,
    ) -> educore_core::error::Result<(ClassPeriod, ClassPeriod)> {
        use educore_core::error::DomainError;
        if period_a_idx >= self.periods.len() || period_b_idx >= self.periods.len() {
            return Err(DomainError::validation(format!(
                "swap indices out of bounds: a={period_a_idx}, b={period_b_idx}, len={}",
                self.periods.len()
            )));
        }
        let (a, b) = (period_a_idx, period_b_idx);
        let prev_a = self.periods[a].clone();
        let prev_b = self.periods[b].clone();
        self.periods.swap(a, b);
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok((prev_a, prev_b))
    }

    /// Soft-delete (retire) the class-routine. Sets
    /// `active_status = Retired` and `is_active = false`,
    /// bumps the audit footer.
    pub fn retire(&mut self, updated_by: UserId, now: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.is_active = false;
        self.updated_at = now;
        self.updated_by = updated_by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}
academic_aggregate_stub! {
    /// An assignment given to students in a class-section, for a
    /// subject, with a submission deadline. See
    /// `docs/specs/academic/aggregates.md` § Homework.
    pub struct Homework { id: HomeworkId }
}
academic_aggregate_stub! {
    /// A teacher's plan for a specific lesson topic on a specific
    /// date. See `docs/specs/academic/aggregates.md` § LessonPlan.
    pub struct LessonPlan { id: LessonPlanId }
}
academic_aggregate_stub! {
    /// A unit of study within a subject, owned by a class-section.
    /// See `docs/specs/academic/aggregates.md` § Lesson.
    pub struct Lesson { id: LessonId }
}
academic_aggregate_stub! {
    /// A topic within a lesson, trackable through a syllabus.
    /// See `docs/specs/academic/aggregates.md` § LessonTopic.
    pub struct LessonTopic { id: LessonTopicId }
}
academic_aggregate_stub! {
    /// A historical record of a promotion event. See
    /// `docs/specs/academic/aggregates.md` § StudentPromotion.
    pub struct StudentPromotion { id: StudentPromotionId }
}
academic_aggregate_stub! {
    /// A categorization for students, used for fee discounts and
    /// reporting. See `docs/specs/academic/aggregates.md` §
    /// StudentCategory.
    pub struct StudentCategory { id: StudentCategoryId }
}
academic_aggregate_stub! {
    /// A grouping of students for non-academic purposes such as
    /// clubs or sports teams. See
    /// `docs/specs/academic/aggregates.md` § StudentGroup.
    pub struct StudentGroup { id: StudentGroupId }
}
academic_aggregate_stub! {
    /// A custom field on the student or staff registration form.
    /// See `docs/specs/academic/aggregates.md` § RegistrationField.
    pub struct RegistrationField { id: RegistrationFieldId }
}
academic_aggregate_stub! {
    /// A configurable certificate template: transfer, character,
    /// course completion, etc. See
    /// `docs/specs/academic/aggregates.md` § Certificate.
    pub struct Certificate { id: CertificateId }
}
academic_aggregate_stub! {
    /// A configurable student ID card template. See
    /// `docs/specs/academic/aggregates.md` § IdCard.
    pub struct IdCard { id: IdCardId }
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
    use educore_core::ids::Identifier;

    fn school() -> SchoolId {
        SchoolId::from_uuid(uuid::Uuid::now_v7())
    }

    fn user() -> UserId {
        UserId::from_uuid(uuid::Uuid::now_v7())
    }

    fn now() -> Timestamp {
        Timestamp::now()
    }

    fn class_id() -> ClassId {
        ClassId::new(school(), uuid::Uuid::now_v7())
    }

    fn section_id() -> SectionId {
        SectionId::new(school(), uuid::Uuid::now_v7())
    }

    fn year_id() -> AcademicYearId {
        AcademicYearId::new(school(), uuid::Uuid::now_v7())
    }

    #[test]
    fn student_fresh_starts_at_initial_version_and_active_status() {
        let id = StudentId::new(school(), uuid::Uuid::now_v7());
        let dob = chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap();
        let admission = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let s = Student::fresh(
            id,
            "ADM-001".to_owned(),
            "Ada".to_owned(),
            "Lovelace".to_owned(),
            dob,
            crate::value_objects::Gender::Female,
            admission,
            class_id(),
            section_id(),
            year_id(),
            Some("1".to_owned()),
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert_eq!(s.version, Version::initial());
        assert_eq!(s.status, crate::value_objects::StudentStatus::Active);
        assert!(s.active_status.is_active());
        assert!(s.last_event_id.is_none());
        assert_eq!(s.school_id, id.school_id());
        assert_eq!(s.full_name(), "Ada Lovelace");
        assert!(s.custom_fields.is_empty());
    }

    #[test]
    fn class_fresh_starts_with_default_gpa_threshold() {
        let id = ClassId::new(school(), uuid::Uuid::now_v7());
        let c = Class::fresh(
            id,
            "Grade 1".to_owned(),
            PassMark::new(50.0).unwrap(),
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert_eq!(c.optional_subject_gpa_threshold.as_f32(), 0.0);
        assert_eq!(c.pass_mark.as_f32(), 50.0);
        assert_eq!(c.school_id, id.school_id());
    }

    #[test]
    fn section_fresh_starts_active() {
        let id = SectionId::new(school(), uuid::Uuid::now_v7());
        let s = Section::fresh(
            id,
            "A".to_owned(),
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert!(s.active_status.is_active());
        assert_eq!(s.school_id, id.school_id());
    }

    #[test]
    fn subject_fresh_round_trip() {
        let id = SubjectId::new(school(), uuid::Uuid::now_v7());
        let s = Subject::fresh(
            id,
            "MATH".to_owned(),
            "Mathematics".to_owned(),
            SubjectType::Theory,
            PassMark::new(40.0).unwrap(),
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert_eq!(s.subject_type, SubjectType::Theory);
        assert_eq!(s.school_id, id.school_id());
    }

    #[test]
    fn academic_year_fresh_starts_not_current_not_closed() {
        let id = year_id();
        let range = AcademicYearRange::new(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        )
        .unwrap();
        let y = AcademicYear::fresh(
            id,
            "2026".to_owned(),
            "Academic Year 2026-2027".to_owned(),
            range,
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert!(!y.is_current);
        assert!(!y.is_closed);
        assert_eq!(y.school_id, id.school_id());
    }

    #[test]
    fn fresh_etag_constants_are_32_lowercase_hex() {
        Etag::new(Student::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
        Etag::new(Class::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
        Etag::new(Section::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
        Etag::new(Subject::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
        Etag::new(AcademicYear::FRESH_ETAG).expect("FRESH_ETAG must be a valid etag");
    }

    #[test]
    fn guardian_fresh_round_trip() {
        let id = GuardianId::new(school(), uuid::Uuid::now_v7());
        let g = Guardian::fresh(
            id,
            "Jane".to_owned(),
            "Doe".to_owned(),
            crate::value_objects::PhoneNumber::new("+14155552671").ok(),
            crate::value_objects::EmailAddress::new("jane@example.com").ok(),
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert_eq!(g.school_id, id.school_id());
        assert_eq!(g.full_name(), "Jane Doe");
        assert!(g.active_status.is_active());
        assert_eq!(g.version, Version::initial());
    }

    #[test]
    fn student_guardian_link_fresh_round_trip() {
        let s = school();
        let gid = GuardianId::new(s, uuid::Uuid::now_v7());
        let sid = StudentId::new(s, uuid::Uuid::now_v7());
        let lid = StudentGuardianLinkId::new(s, uuid::Uuid::now_v7());
        let l = StudentGuardianLink::fresh(
            lid,
            gid,
            sid,
            crate::value_objects::Relation::Father,
            true,
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert_eq!(l.school_id, s);
        assert_eq!(l.guardian_id, gid);
        assert_eq!(l.student_id, sid);
        assert!(l.is_primary);
        assert_eq!(l.relation, crate::value_objects::Relation::Father);
    }

    #[test]
    fn optional_subject_assignment_fresh_round_trip() {
        let s = school();
        let sid = StudentId::new(s, uuid::Uuid::now_v7());
        let subj = SubjectId::new(s, uuid::Uuid::now_v7());
        let yid = AcademicYearId::new(s, uuid::Uuid::now_v7());
        let aid = OptionalSubjectAssignmentId::new(s, uuid::Uuid::now_v7());
        let a = OptionalSubjectAssignment::fresh(
            aid,
            sid,
            subj,
            yid,
            user(),
            user(),
            now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        );
        assert_eq!(a.school_id, s);
        assert_eq!(a.student_id, sid);
        assert_eq!(a.subject_id, subj);
        assert_eq!(a.academic_year_id, yid);
    }
}

// =============================================================================
// StudentRecord (Cluster D mop-up)
// =============================================================================

/// Per-school enrollment record for a student.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentRecord {
    pub id: StudentRecordId,
    pub school_id: SchoolId,
}
