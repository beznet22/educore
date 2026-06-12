//! # Assessment-domain child entities
//!
//! Per-aggregate children that are not part of the aggregate
//! root's struct but are owned by it (e.g. a per-subject
//! `MarksRegisterChild` row belongs to a `MarksRegister`).
//!
//! Phase 4 Workstream A does not yet ship any child entities
//! — the `Exam` aggregate is a single root with no children.
//! Workstreams B, C, and D add the child types for
//! `ExamScheduleSubject` (B), `MarksRegisterChild` (C),
//! `OnlineExam*` (D), `SeatPlanChild` (B), and the question
//! child types (D). The file is kept as part of the 9-file
//! module layout so the workstream-B/C/D commits can extend
//! it without touching `lib.rs`.

// Workstreams B, C, D will add child entity types here.
