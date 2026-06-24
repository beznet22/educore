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

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    AcademicYearId, AcademicYearRange, CertificateId, ClassId, ClassRoutineId, ClassSectionId,
    ClassSubjectId, GuardianId, HomeworkId, IdCardId, LessonId, LessonPlanId, LessonTopicId,
    OptionalSubjectGpaThreshold, PassMark, RegistrationFieldId, SectionId, StudentCategoryId,
    StudentGroupId, StudentId, StudentPromotionId, StudentRecordId, SubjectId, SubjectType,
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

academic_aggregate_stub! {
    /// A parent, legal guardian, or authorized contact for a
    /// [`Student`]. See
    /// `docs/specs/academic/aggregates.md` § Guardian.
    pub struct Guardian { id: GuardianId }
}
academic_aggregate_stub! {
    /// The pairing of a class and a section in a specific
    /// academic year that students are enrolled into. See
    /// `docs/specs/academic/aggregates.md` § ClassSection.
    pub struct ClassSection { id: ClassSectionId }
}
academic_aggregate_stub! {
    /// The assignment of a subject to a class, with a teacher,
    /// in a specific academic year. See
    /// `docs/specs/academic/aggregates.md` § ClassSubject.
    pub struct ClassSubject { id: ClassSubjectId }
}
academic_aggregate_stub! {
    /// The weekly schedule for a class-section-subject
    /// combination. See `docs/specs/academic/aggregates.md` §
    /// ClassRoutine.
    pub struct ClassRoutine { id: ClassRoutineId }
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
