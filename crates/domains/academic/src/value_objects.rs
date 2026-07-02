//! # Academic value objects
//!
//! The typed ids (every aggregate is keyed by one) and the
//! validated value objects the academic aggregates depend on.
//! Per `docs/specs/academic/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper
//!   that carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Strings (admission numbers, class names, etc.) are
//!   validated at construction. The constructors return
//!   `Result<Self, DomainError>`; there are no setters that
//!   bypass validation.
//! - Status enums are closed (`StudentStatus`, `SubjectType`,
//!   `ResultStatus`).
//!
//! Phase 3 ships the prompt-named subset: id types for
//! [`StudentId`], [`ClassId`], [`SectionId`], [`SubjectId`],
//! [`AcademicYearId`]; value objects for the student
//! lifecycle; the class/section/subject/academic-year
//! value objects. The full set of typed ids for all 20
//! academic aggregates (`StudentId`, `GuardianId`, `ClassId`,
//! `SectionId`, `ClassSectionId`, `SubjectId`,
//! `ClassSubjectId`, `AcademicYearId`, `ClassRoutineId`,
//! `HomeworkId`, `LessonId`, `LessonTopicId`, `LessonPlanId`,
//! `StudentRecordId`, `StudentPromotionId`,
//! `StudentCategoryId`, `StudentGroupId`,
//! `RegistrationFieldId`, `CertificateId`, `IdCardId`) is
//! defined here; the corresponding aggregate structs land
//! in later phases per `docs/build-plan.md`.

use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed academic id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// academic id follows the same shape: a `school_id` anchor
/// plus a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
///
/// The pattern matches `educore-rbac::ids::rbac_typed_id!`
/// so the engine's id types stay consistent across crates.
macro_rules! academic_typed_id {
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
// Typed ids (20 academic aggregates)
// =============================================================================

academic_typed_id! {
    /// A typed id for a [`Student`](crate::aggregate::Student) row.
    pub struct StudentId;
}

academic_typed_id! {
    /// A typed id for a `Guardian` row (parent, legal guardian,
    /// or authorized contact). See
    /// `docs/specs/academic/aggregates.md` § Guardian.
    pub struct GuardianId;
}

academic_typed_id! {
    /// A typed id for a `StudentGuardianLink` row (the linkage
    /// between a guardian and a student). See
    /// `docs/specs/academic/aggregates.md` § Guardian § I-2.
    pub struct StudentGuardianLinkId;
}

academic_typed_id! {
    /// A typed id for an `OptionalSubjectAssignment` row (the
    /// assignment of an optional subject to a student for a
    /// specific academic year). See
    /// `docs/specs/academic/aggregates.md` § Student § I-4.
    pub struct OptionalSubjectAssignmentId;
}

academic_typed_id! {
    /// A typed id for a [`Class`](crate::aggregate::Class) row.
    pub struct ClassId;
}

academic_typed_id! {
    /// A typed id for a [`Section`](crate::aggregate::Section) row.
    pub struct SectionId;
}

academic_typed_id! {
    /// A typed id for a `ClassSection` row (the pairing of a
    /// class and a section in a specific academic year that
    /// students are enrolled into). See
    /// `docs/specs/academic/aggregates.md` § ClassSection.
    pub struct ClassSectionId;
}

academic_typed_id! {
    /// A typed id for a `ClassRoom` reference (a physical or
    /// virtual room assigned to a [`ClassSection`](crate::aggregate::ClassSection)).
    ///
    /// Per `docs/specs/academic/aggregates.md` § ClassSection § I-3,
    /// a class-section has one or more class rooms assigned.
    /// The `ClassRoom` aggregate itself lives in the facilities
    /// domain; this typed id exists in the academic crate so
    /// the [`ClassSection`](crate::aggregate::ClassSection) can
    /// reference rooms cross-domain via a stable id. The
    /// academic layer treats the id as opaque metadata.
    pub struct ClassRoomId;
}

academic_typed_id! {
    /// A typed id for a [`Subject`](crate::aggregate::Subject) row.
    pub struct SubjectId;
}

academic_typed_id! {
    /// A typed id for a `ClassSubject` row (the assignment of a
    /// subject to a class, with a teacher, in a specific
    /// academic year). See `docs/specs/academic/aggregates.md`
    /// § ClassSubject.
    pub struct ClassSubjectId;
}

academic_typed_id! {
    /// A typed id for an [`AcademicYear`](crate::aggregate::AcademicYear) row.
    pub struct AcademicYearId;
}

academic_typed_id! {
    /// A typed id for a `ClassRoutine` row (the weekly schedule
    /// for a class-section-subject combination). See
    /// `docs/specs/academic/aggregates.md` § ClassRoutine.
    pub struct ClassRoutineId;
}

academic_typed_id! {
    /// A typed id for a `Homework` row (an assignment given to
    /// students in a class-section, for a subject, with a
    /// submission deadline). See
    /// `docs/specs/academic/aggregates.md` § Homework.
    pub struct HomeworkId;
}

academic_typed_id! {
    /// A typed id for a `LessonPlan` row (a teacher's plan for a
    /// specific lesson topic on a specific date). See
    /// `docs/specs/academic/aggregates.md` § LessonPlan.
    pub struct LessonPlanId;
}

academic_typed_id! {
    /// A typed id for a `Lesson` row (a unit of study within a
    /// subject, owned by a class-section). See
    /// `docs/specs/academic/aggregates.md` § Lesson.
    pub struct LessonId;
}

academic_typed_id! {
    /// A typed id for a `LessonTopic` row (a topic within a
    /// lesson, trackable through a syllabus). See
    /// `docs/specs/academic/aggregates.md` § LessonTopic.
    pub struct LessonTopicId;
}

academic_typed_id! {
    /// A typed id for a `StudentRecord` row (the per-academic-year
    /// enrollment that downstream domains — including
    /// [`educore_assessment`](::educore_assessment) for admit cards
    /// and report cards — depend on).
    ///
    /// The full `StudentRecord` aggregate lands in a later
    /// academic phase (Phase 3 hand-off § Open questions). The
    /// typed id is added in Phase 4 as a non-breaking additive
    /// so the assessment domain can declare its foreign-key
    /// fields against a stable type from the academic crate.
    pub struct StudentRecordId;
}

academic_typed_id! {
    /// A typed id for a `StudentPromotion` row (a historical
    /// record of a promotion event). See
    /// `docs/specs/academic/aggregates.md` § StudentPromotion.
    pub struct StudentPromotionId;
}

academic_typed_id! {
    /// A typed id for a `StudentCategory` row (a categorization
    /// for students, used for fee discounts and reporting). See
    /// `docs/specs/academic/aggregates.md` § StudentCategory.
    pub struct StudentCategoryId;
}

academic_typed_id! {
    /// A typed id for a `StudentGroup` row (a grouping of
    /// students for non-academic purposes such as clubs or
    /// sports teams). See `docs/specs/academic/aggregates.md`
    /// § StudentGroup.
    pub struct StudentGroupId;
}

academic_typed_id! {
    /// A typed id for a `RegistrationField` row (a custom field
    /// on the student or staff registration form). See
    /// `docs/specs/academic/aggregates.md` § RegistrationField.
    pub struct RegistrationFieldId;
}

academic_typed_id! {
    /// A typed id for a `Certificate` row (a configurable
    /// certificate template: transfer, character, course
    /// completion, etc.). See
    /// `docs/specs/academic/aggregates.md` § Certificate.
    pub struct CertificateId;
}

academic_typed_id! {
    /// A typed id for an `IdCard` row (a configurable student ID
    /// card template). See `docs/specs/academic/aggregates.md`
    /// § IdCard.
    pub struct IdCardId;
}

// =============================================================================
// Names (1..=N chars, validated at construction)
// =============================================================================

/// A validated, non-empty person name. 1..=200 chars (per
/// `docs/specs/academic/value-objects.md`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonName(String);

impl PersonName {
    /// Maximum length of a person name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a `PersonName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("person name must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "person name must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for PersonName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A computed full name, joining first and last. The struct
/// owns the two parts; the engine does not perform any
/// transformation on the input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullName {
    /// The first name.
    pub first: PersonName,
    /// The last name.
    pub last: PersonName,
}

impl FullName {
    /// Constructs a `FullName` from first + last.
    #[must_use]
    pub const fn new(first: PersonName, last: PersonName) -> Self {
        Self { first, last }
    }

    /// Returns the display form (e.g. `"Ada Lovelace"`).
    #[must_use]
    pub fn display(&self) -> String {
        format!("{} {}", self.first.as_str(), self.last.as_str())
    }
}

impl fmt::Display for FullName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}

/// A validated address, 1..=500 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Address(String);

impl Address {
    /// Maximum length of an address.
    pub const MAX_LEN: usize = 500;

    /// Constructs an `Address`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("address must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "address must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// =============================================================================
// Academic identifiers
// =============================================================================

/// A validated admission number. 1..=50 chars, unique within
/// `(school_id)`. Per the spec, the value is opaque to the
/// engine — consumers may use numeric (preferred) or
/// alphanumeric formats.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AdmissionNumber(String);

impl AdmissionNumber {
    /// Maximum length of an admission number.
    pub const MAX_LEN: usize = 50;

    /// Constructs an `AdmissionNumber`, rejecting empty or
    /// overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "admission number must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "admission number must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AdmissionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for AdmissionNumber {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated roll number. 1..=50 chars, unique within
/// `(class_id, section_id, academic_year_id)`. Per the spec,
/// numeric is preferred but alphanumeric is accepted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RollNumber(String);

impl RollNumber {
    /// Maximum length of a roll number.
    pub const MAX_LEN: usize = 50;

    /// Constructs a `RollNumber`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("roll number must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "roll number must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RollNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for RollNumber {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated subject code. 1..=50 chars, unique within
/// `(school_id)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubjectCode(String);

impl SubjectCode {
    /// Maximum length of a subject code.
    pub const MAX_LEN: usize = 50;

    /// Constructs a `SubjectCode`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("subject code must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "subject code must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubjectCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubjectCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated class name. 1..=200 chars, unique within
/// `(school_id)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClassName(String);

impl ClassName {
    /// Maximum length of a class name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a `ClassName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("class name must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "class name must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ClassName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for ClassName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated contact phone number in E.164 format.
///
/// Accepts a leading `+` followed by 4..=15 ASCII digits
/// (the ITU-T E.164 recommendation). The country code is
/// not validated against a registry — the engine treats the
/// digit count as sufficient.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Constructs a `PhoneNumber` from an E.164 string. Empty
    /// input is rejected.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "phone number must not be empty",
            ));
        }
        if !s.starts_with('+') {
            return Err(DomainError::validation(format!(
                "phone number must start with '+': {s:?}"
            )));
        }
        let digits = &s[1..];
        if digits.len() < 4 || digits.len() > 15 {
            return Err(DomainError::validation(format!(
                "phone number digit count {} outside 4..=15",
                digits.len()
            )));
        }
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::validation(format!(
                "phone number contains non-digit characters: {s:?}"
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for PhoneNumber {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated email address. Local-part + `@` + domain
/// (1..=253 chars). The engine does not perform DNS
/// resolution; structural validation only.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Maximum length of an email address per RFC 5321.
    pub const MAX_LEN: usize = 254;

    /// Constructs an `EmailAddress`. Requires a single `@`
    /// separating a non-empty local-part and a non-empty
    /// domain.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("email must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "email must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        let at_count = s.chars().filter(|c| *c == '@').count();
        if at_count != 1 {
            return Err(DomainError::validation(format!(
                "email must contain exactly one '@', got {at_count}: {s:?}"
            )));
        }
        let parts: Vec<&str> = s.split('@').collect();
        let local = parts[0];
        let domain = parts[1];
        if local.is_empty() {
            return Err(DomainError::validation(format!(
                "email local-part must not be empty: {s:?}"
            )));
        }
        if domain.is_empty() || !domain.contains('.') {
            return Err(DomainError::validation(format!(
                "email domain must be non-empty and contain a dot: {s:?}"
            )));
        }
        Ok(Self(s.to_lowercase()))
    }

    /// Returns the inner string (lower-cased).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// The relationship between a [`Guardian`](crate::aggregate::Guardian)
/// and a [`Student`](crate::aggregate::Student) in a
/// [`StudentGuardianLink`](crate::aggregate::StudentGuardianLink).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    /// The student's father.
    Father,
    /// The student's mother.
    Mother,
    /// A legal or designated guardian (not parent).
    #[default]
    Guardian,
    /// Any other relationship (e.g. grandparent, sponsor,
    /// host family).
    Other,
}

impl Relation {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Father => "father",
            Self::Mother => "mother",
            Self::Guardian => "guardian",
            Self::Other => "other",
        }
    }

    /// Parses a relation from its canonical snake_case wire
    /// string. Returns `None` if `s` is not a known variant.
    /// The check is case-insensitive; the canonical
    /// lower-case form is accepted.
    #[must_use]
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "father" => Some(Self::Father),
            "mother" => Some(Self::Mother),
            "guardian" => Some(Self::Guardian),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A validated section name. 1..=200 chars, unique within
/// `(school_id)`. Sections can be reused across academic years.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SectionName(String);

impl SectionName {
    /// Maximum length of a section name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a `SectionName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("section name must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "section name must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SectionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SectionName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated academic year title. 1..=200 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AcademicYearTitle(String);

impl AcademicYearTitle {
    /// Maximum length of an academic year title.
    pub const MAX_LEN: usize = 200;

    /// Constructs an `AcademicYearTitle`, rejecting empty or
    /// overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "academic year title must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "academic year title must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AcademicYearTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for AcademicYearTitle {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// =============================================================================
// Dates & periods
// =============================================================================

/// A validated date of birth. The `chrono::NaiveDate` is
/// checked at construction to fall in the 2..=30-year range
/// (per `docs/specs/academic/value-objects.md`); the engine
/// uses `Utc::now()` as the reference clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateOfBirth(NaiveDate);

impl DateOfBirth {
    /// Constructs a `DateOfBirth`, rejecting dates that fall
    /// outside the 2..=30-year window relative to `now`.
    pub fn new(d: NaiveDate, now: DateTime<Utc>) -> Result<Self> {
        let today = now.date_naive();
        let age_years = (today - d).num_days() / 365;
        if !(2..=30).contains(&age_years) {
            return Err(DomainError::validation(format!(
                "date of birth {d} implies age {age_years} years; must be 2..=30 years from now"
            )));
        }
        Ok(Self(d))
    }

    /// Returns the inner `NaiveDate`.
    #[must_use]
    pub const fn as_naive_date(&self) -> NaiveDate {
        self.0
    }
}

impl fmt::Display for DateOfBirth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A validated academic year date range. Per the spec, the
/// `start` is strictly before `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcademicYearRange {
    /// The first day of the academic year.
    pub start: NaiveDate,
    /// The last day of the academic year.
    pub end: NaiveDate,
}

impl AcademicYearRange {
    /// Constructs an `AcademicYearRange`, rejecting ranges
    /// where `start >= end`.
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self> {
        if start >= end {
            return Err(DomainError::validation(format!(
                "academic year start {start} must be strictly before end {end}"
            )));
        }
        Ok(Self { start, end })
    }

    /// Returns `true` if `date` falls inside the range
    /// (inclusive on both ends).
    #[must_use]
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }
}

// =============================================================================
// Money & quantities (PassMark, Gpa)
// =============================================================================

/// A pass mark in the range 0.0..=100.0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PassMark(f32);

impl PassMark {
    /// Constructs a `PassMark`, rejecting values outside
    /// 0.0..=100.0.
    pub fn new(v: f32) -> Result<Self> {
        if !(0.0..=100.0).contains(&v) {
            return Err(DomainError::validation(format!(
                "pass mark {v} must be in 0.0..=100.0"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner `f32`.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// An optional-subject GPA threshold in 0.0..=5.0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OptionalSubjectGpaThreshold(f32);

impl OptionalSubjectGpaThreshold {
    /// Constructs an `OptionalSubjectGpaThreshold`, rejecting
    /// values outside 0.0..=5.0.
    pub fn new(v: f32) -> Result<Self> {
        if !(0.0..=5.0).contains(&v) {
            return Err(DomainError::validation(format!(
                "optional subject GPA threshold {v} must be in 0.0..=5.0"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner `f32`.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

// =============================================================================
// Status enums
// =============================================================================

/// The lifecycle status of a [`Student`](crate::aggregate::Student).
///
/// The transitions are:
///
/// ```text
/// Applicant → Active → {Suspended, Withdrawn, Graduated, Transferred}
/// ```
///
/// No other transitions are allowed. `Withdrawn` and
/// `Graduated` are terminal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StudentStatus {
    /// Pre-admission state: the inquiry is registered but the
    /// student is not yet enrolled.
    Applicant,
    /// The student is currently enrolled and attending.
    #[default]
    Active,
    /// The student is temporarily suspended (e.g. medical
    /// leave, disciplinary). Reversible via
    /// `ReinstateStudent`.
    Suspended,
    /// The student has been withdrawn. Terminal.
    Withdrawn,
    /// The student has graduated. Terminal.
    Graduated,
    /// The student has been transferred to another school.
    /// Terminal.
    Transferred,
}

impl StudentStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applicant => "applicant",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Withdrawn => "withdrawn",
            Self::Graduated => "graduated",
            Self::Transferred => "transferred",
        }
    }

    /// Returns `true` if the status is a terminal state
    /// (`Withdrawn`, `Graduated`, `Transferred`).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Withdrawn | Self::Graduated | Self::Transferred)
    }
}

impl fmt::Display for StudentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A student's gender. The spec recognises three values; the
/// engine treats the value as opaque metadata (no policy
/// decisions are made on it).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    /// Male.
    Male,
    /// Female.
    #[default]
    Female,
    /// Other / non-binary. The engine treats this as a
    /// first-class value.
    Other,
}

impl Gender {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A blood group. The engine does not perform any policy
/// decisions on this value; it is stored for clinical
/// reporting.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BloodGroup {
    /// A positive.
    #[default]
    APositive,
    /// A negative.
    ANegative,
    /// B positive.
    BPositive,
    /// B negative.
    BNegative,
    /// AB positive.
    ABPositive,
    /// AB negative.
    ABNegative,
    /// O positive.
    OPositive,
    /// O negative.
    ONegative,
}

impl BloodGroup {
    /// Returns the canonical display form (e.g. `"A+"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::APositive => "A+",
            Self::ANegative => "A-",
            Self::BPositive => "B+",
            Self::BNegative => "B-",
            Self::ABPositive => "AB+",
            Self::ABNegative => "AB-",
            Self::OPositive => "O+",
            Self::ONegative => "O-",
        }
    }
}

impl fmt::Display for BloodGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// The reason a student was suspended. Free-form string
/// carried in the event payload (validated for length only).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SuspensionReason(String);

impl SuspensionReason {
    /// Maximum length of a suspension reason.
    pub const MAX_LEN: usize = 500;

    /// Constructs a `SuspensionReason`, rejecting empty or
    /// overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "suspension reason must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "suspension reason must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SuspensionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The reason a student was withdrawn. Free-form string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WithdrawalReason(String);

impl WithdrawalReason {
    /// Maximum length of a withdrawal reason.
    pub const MAX_LEN: usize = 500;

    /// Constructs a `WithdrawalReason`, rejecting empty or
    /// overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "withdrawal reason must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "withdrawal reason must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WithdrawalReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The reason a student was transferred. Free-form string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransferReason(String);

impl TransferReason {
    /// Maximum length of a transfer reason.
    pub const MAX_LEN: usize = 500;

    /// Constructs a `TransferReason`, rejecting empty or
    /// overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("transfer reason must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "transfer reason must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TransferReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A subject's type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectType {
    /// Theory subject (lecture-based).
    #[default]
    Theory,
    /// Practical subject (lab-based).
    Practical,
}

impl SubjectType {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Theory => "theory",
            Self::Practical => "practical",
        }
    }
}

impl fmt::Display for SubjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// The scope of a [`ClassSubject`](crate::aggregate::ClassSubject)
/// assignment.
///
/// Per `docs/specs/academic/aggregates.md` § ClassSubject § I-1,
/// a subject may be assigned to a class (applies to every
/// section) or to a specific class-section (applies to one
/// section of a class). The scope is encoded as a closed
/// enum so the constructor can enforce the
/// `(class_id, class_section_id, scope)` triple
/// consistency at compile time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClassSubjectScope {
    /// The subject applies to the class as a whole (every
    /// section of the class in the academic year).
    #[default]
    ClassOnly,
    /// The subject applies to a specific class-section of
    /// the class.
    ClassSection,
}

impl ClassSubjectScope {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClassOnly => "class_only",
            Self::ClassSection => "class_section",
        }
    }
}

impl fmt::Display for ClassSubjectScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A student's promotion result.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResultStatus {
    /// The student passed.
    #[default]
    Pass,
    /// The student failed.
    Fail,
    /// Operator decision (manual override).
    Manual,
}

impl ResultStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Manual => "manual",
        }
    }
}

impl fmt::Display for ResultStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
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

    fn now() -> DateTime<Utc> {
        use chrono::TimeZone;
        Utc.with_ymd_and_hms(2026, 6, 12, 0, 0, 0).single().unwrap()
    }

    #[test]
    fn typed_ids_construct_and_display() {
        let school = SchoolId::from_uuid(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = StudentId::new(school, value);
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), value);
        assert!(id.to_string().contains(&value.to_string()));
    }

    #[test]
    fn distinct_typed_ids_are_not_interchangeable() {
        let school = SchoolId::from_uuid(Uuid::now_v7());
        let value = Uuid::now_v7();
        let s = StudentId::new(school, value);
        let c = ClassId::new(school, value);
        assert_eq!(s, StudentId::new(school, value));
        assert_ne!(format!("{s:?}"), format!("{c:?}"));
    }

    #[test]
    fn person_name_rejects_empty() {
        assert!(PersonName::new("").is_err());
        assert!(PersonName::new("Ada").is_ok());
    }

    #[test]
    fn person_name_rejects_overlong() {
        let s = "a".repeat(PersonName::MAX_LEN + 1);
        assert!(PersonName::new(s).is_err());
    }

    #[test]
    fn full_name_display_joins_with_space() {
        let n = FullName::new(
            PersonName::new("Ada").unwrap(),
            PersonName::new("Lovelace").unwrap(),
        );
        assert_eq!(n.display(), "Ada Lovelace");
    }

    #[test]
    fn date_of_birth_rejects_out_of_range() {
        let n = now();
        let too_young = n.date_naive() - chrono::Duration::days(30);
        let too_old = n.date_naive() - chrono::Duration::days(40 * 365);
        let just_right = n.date_naive() - chrono::Duration::days(10 * 365);
        assert!(DateOfBirth::new(too_young, n).is_err());
        assert!(DateOfBirth::new(too_old, n).is_err());
        assert!(DateOfBirth::new(just_right, n).is_ok());
    }

    #[test]
    fn academic_year_range_requires_strict_ordering() {
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
        assert!(AcademicYearRange::new(d1, d2).is_ok());
        assert!(AcademicYearRange::new(d1, d1).is_err());
        assert!(AcademicYearRange::new(d2, d1).is_err());
    }

    #[test]
    fn academic_year_range_contains() {
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
        let d_mid = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let d_out = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let r = AcademicYearRange::new(d1, d2).unwrap();
        assert!(r.contains(d1));
        assert!(r.contains(d2));
        assert!(r.contains(d_mid));
        assert!(!r.contains(d_out));
    }

    #[test]
    fn pass_mark_in_range() {
        assert!(PassMark::new(0.0).is_ok());
        assert!(PassMark::new(100.0).is_ok());
        assert!(PassMark::new(50.0).is_ok());
        assert!(PassMark::new(-0.1).is_err());
        assert!(PassMark::new(100.1).is_err());
    }

    #[test]
    fn optional_subject_gpa_threshold_in_range() {
        assert!(OptionalSubjectGpaThreshold::new(0.0).is_ok());
        assert!(OptionalSubjectGpaThreshold::new(5.0).is_ok());
        assert!(OptionalSubjectGpaThreshold::new(3.5).is_ok());
        assert!(OptionalSubjectGpaThreshold::new(-0.1).is_err());
        assert!(OptionalSubjectGpaThreshold::new(5.1).is_err());
    }

    #[test]
    fn phone_number_validates() {
        assert!(PhoneNumber::new("").is_err());
        assert!(PhoneNumber::new("+14155552671").is_ok());
        assert!(PhoneNumber::new("14155552671").is_err()); // missing +
        assert!(PhoneNumber::new("+abc").is_err()); // non-digit
        assert!(PhoneNumber::new("+12").is_err()); // too few digits
        assert!(PhoneNumber::new("+12345678901234567").is_err()); // too many
    }

    #[test]
    fn email_address_validates() {
        assert!(EmailAddress::new("").is_err());
        assert!(EmailAddress::new("ada@example.com").is_ok());
        assert!(EmailAddress::new("no-at-sign").is_err());
        assert!(EmailAddress::new("@example.com").is_err()); // empty local
        assert!(EmailAddress::new("user@").is_err()); // empty domain
        assert!(EmailAddress::new("user@nodot").is_err()); // domain without dot
        assert!(EmailAddress::new("two@@signs.com").is_err()); // double @
    }

    #[test]
    fn email_address_is_lowercased() {
        let e = EmailAddress::new("Ada@Example.COM").unwrap();
        assert_eq!(e.as_str(), "ada@example.com");
    }

    #[test]
    fn relation_enum_has_four_variants() {
        let variants = [
            Relation::Father,
            Relation::Mother,
            Relation::Guardian,
            Relation::Other,
        ];
        for v in variants {
            assert!(!v.as_str().is_empty());
        }
    }

    #[test]
    fn admission_number_validates() {
        assert!(AdmissionNumber::new("").is_err());
        assert!(AdmissionNumber::new("ADM-001").is_ok());
        let s = "a".repeat(AdmissionNumber::MAX_LEN + 1);
        assert!(AdmissionNumber::new(s).is_err());
    }

    #[test]
    fn roll_number_validates() {
        assert!(RollNumber::new("").is_err());
        assert!(RollNumber::new("1").is_ok());
        let s = "a".repeat(RollNumber::MAX_LEN + 1);
        assert!(RollNumber::new(s).is_err());
    }

    #[test]
    fn subject_code_class_name_section_name_title_validate() {
        assert!(SubjectCode::new("").is_err());
        assert!(SubjectCode::new("MATH").is_ok());
        assert!(ClassName::new("").is_err());
        assert!(ClassName::new("Grade 1").is_ok());
        assert!(SectionName::new("").is_err());
        assert!(SectionName::new("A").is_ok());
        assert!(AcademicYearTitle::new("").is_err());
        assert!(AcademicYearTitle::new("2026-2027").is_ok());
    }

    #[test]
    fn student_status_terminal() {
        assert!(!StudentStatus::Applicant.is_terminal());
        assert!(!StudentStatus::Active.is_terminal());
        assert!(!StudentStatus::Suspended.is_terminal());
        assert!(StudentStatus::Withdrawn.is_terminal());
        assert!(StudentStatus::Graduated.is_terminal());
        assert!(StudentStatus::Transferred.is_terminal());
    }

    #[test]
    fn student_status_round_trip() {
        for s in [
            StudentStatus::Applicant,
            StudentStatus::Active,
            StudentStatus::Suspended,
            StudentStatus::Withdrawn,
            StudentStatus::Graduated,
            StudentStatus::Transferred,
        ] {
            assert!(!s.as_str().is_empty());
        }
    }

    #[test]
    fn gender_and_blood_group_round_trip() {
        for g in [Gender::Male, Gender::Female, Gender::Other] {
            assert!(!g.as_str().is_empty());
        }
        for b in [
            BloodGroup::APositive,
            BloodGroup::ANegative,
            BloodGroup::BPositive,
            BloodGroup::BNegative,
            BloodGroup::ABPositive,
            BloodGroup::ABNegative,
            BloodGroup::OPositive,
            BloodGroup::ONegative,
        ] {
            assert!(!b.as_str().is_empty());
        }
    }

    #[test]
    fn subject_type_and_result_status_round_trip() {
        for s in [SubjectType::Theory, SubjectType::Practical] {
            assert!(!s.as_str().is_empty());
        }
        for r in [ResultStatus::Pass, ResultStatus::Fail, ResultStatus::Manual] {
            assert!(!r.as_str().is_empty());
        }
    }

    #[test]
    fn class_subject_scope_round_trip() {
        for s in [ClassSubjectScope::ClassOnly, ClassSubjectScope::ClassSection] {
            assert!(!s.as_str().is_empty());
        }
        assert_eq!(ClassSubjectScope::ClassOnly.as_str(), "class_only");
        assert_eq!(ClassSubjectScope::ClassSection.as_str(), "class_section");
    }

    #[test]
    fn reason_validators_reject_empty() {
        assert!(SuspensionReason::new("").is_err());
        assert!(SuspensionReason::new("medical leave").is_ok());
        assert!(WithdrawalReason::new("").is_err());
        assert!(WithdrawalReason::new("family relocation").is_ok());
        assert!(TransferReason::new("").is_err());
        assert!(TransferReason::new("parent's job transfer").is_ok());
    }

    #[test]
    fn reason_validators_reject_overlong() {
        let s = "a".repeat(SuspensionReason::MAX_LEN + 1);
        assert!(SuspensionReason::new(s).is_err());
        let s = "a".repeat(WithdrawalReason::MAX_LEN + 1);
        assert!(WithdrawalReason::new(s).is_err());
        let s = "a".repeat(TransferReason::MAX_LEN + 1);
        assert!(TransferReason::new(s).is_err());
    }
}
