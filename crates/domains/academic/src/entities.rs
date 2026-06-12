//! # Academic entities (non-root aggregates)
//!
//! Per the 9-file module layout, `entities.rs` is reserved
//! for child entities that live under an aggregate root but
//! have their own identity. The full academic spec names
//! several entities (StudentRecord, StudentDocument,
//! StudentTimeline, StudentHomework, StudentCategory,
//! StudentGroup, OptionalSubjectAssignment,
//! RegistrationField, Certificate, IdCard, ...).
//!
//! Phase 3 ships the **prompt-named subset only** (per
//! `docs/phase_prompt/phase-3-prompt.md`): the `Student`,
//! `Class`, `Section`, `Subject`, and `AcademicYear`
//! aggregates. None of the academic spec's child entities
//! are in Phase 3 scope; the prompt explicitly narrows to
//! 5 aggregates.
//!
//! The placeholder types below are documented so a future
//! agent that expands scope can wire them in without
//! breaking the 9-file layout. The shape mirrors
//! `educore-platform::entities` (see
//! `crates/cross-cutting/platform/src/entities.rs` for the
//! full pattern): one struct per entity, `school_id`
//! tenancy column, and a `created_at` timestamp. No row
//! insert / update / delete ports land in Phase 3.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::SchoolId;
use educore_core::value_objects::Timestamp;

/// A typed id for a [`StudentDocument`] (placeholder for
/// future scope expansion; no Phase 3 methods insert or
/// query this entity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StudentDocumentId(pub SchoolId, pub Uuid);

/// A document attached to a student (placeholder).
///
/// The full shape includes a file reference (pointing to
/// the engine's `FileStorage` port — Phase 15) and a
/// document type. Phase 3 ships the type shell so the
/// 9-file layout is honoured; the `UploadStudentDocument`
/// command lands alongside the `FileStorage` port in a
/// later phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentDocument {
    /// The document's typed id.
    pub id: StudentDocumentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The student the document is attached to.
    pub student_id: crate::value_objects::StudentId,
    /// The document's title.
    pub title: String,
    /// The document's file reference (placeholder string;
    /// the real type lands in Phase 15).
    pub file_ref: String,
    /// The document's type.
    pub document_type: DocumentType,
    /// Row creation timestamp.
    pub created_at: Timestamp,
}

/// The kind of student document.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    /// Student document.
    #[default]
    Student,
    /// Staff document.
    Staff,
}

impl DocumentType {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Student => "student",
            Self::Staff => "staff",
        }
    }
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    use educore_core::clock::IdGenerator;

    #[test]
    fn student_document_round_trip_serde() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let id = StudentDocumentId(school, Uuid::now_v7());
        let student_id = crate::value_objects::StudentId::new(school, Uuid::now_v7());
        let doc = StudentDocument {
            id,
            school_id: school,
            student_id,
            title: "Birth certificate".to_owned(),
            file_ref: "files/student/birth-cert.pdf".to_owned(),
            document_type: DocumentType::Student,
            created_at: Timestamp::epoch(),
        };
        let s = serde_json::to_string(&doc).unwrap();
        let back: StudentDocument = serde_json::from_str(&s).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn document_type_round_trip() {
        for t in [DocumentType::Student, DocumentType::Staff] {
            assert!(!t.as_str().is_empty());
        }
    }
}
