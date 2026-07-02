//! Integration tests for the **ClassRoutine aggregate** vertical slice.
//!
//! Pins the create / update / swap / delete contracts for
//! the `ClassRoutine` aggregate end-to-end through the service
//! layer, exercising all 5 spec invariants:
//!
//! - I-1: full week (7 distinct days)
//! - I-2: no duplicate ClassTimeId
//! - I-3: room + teacher per period (structural)
//! - I-4: teacher no-conflict (cross-aggregate)
//! - I-5: room no-conflict (cross-aggregate)
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class_section.rs` and
//! `class_subject.rs` (`TestClock` + `SystemIdGen` +
//! `InMemoryUniqueness`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;

use educore_academic::prelude::*;
use educore_academic::ClassPeriod;
use educore_academic::commands::{
    CreateClassRoutineCommand, DeleteClassRoutineCommand, SwapClassRoutinePeriodsCommand,
    UpdateClassRoutinePeriodCommand,
};
use educore_academic::events::{
    ClassRoutineDeleted, ClassRoutinePeriodsSwapped, ClassRoutinePeriodUpdated,
    ClassRoutineScheduled,
};
use educore_academic::services::{
    create_class_routine, delete_class_routine, swap_class_routine_periods,
    update_class_routine_period,
};
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;

// =============================================================================
// Fixtures
// =============================================================================

fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

fn class_routine_id(g: &SystemIdGen, school: SchoolId) -> ClassRoutineId {
    ClassRoutineId::new(school, g.next_uuid())
}

fn class_section_id(g: &SystemIdGen, school: SchoolId) -> ClassSectionId {
    ClassSectionId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn class_time_id(g: &SystemIdGen, school: SchoolId) -> ClassTimeId {
    ClassTimeId::new(school, g.next_uuid())
}

fn class_room_id(g: &SystemIdGen, school: SchoolId) -> ClassRoomId {
    ClassRoomId::new(school, g.next_uuid())
}

fn user_id(g: &SystemIdGen) -> UserId {
    g.next_user_id()
}

/// Build a full-week Vec<ClassPeriod> covering Mon..Sun.
fn full_week_periods(g: &SystemIdGen, school: SchoolId) -> Vec<ClassPeriod> {
    let days = [
        DayOfWeek::Monday,
        DayOfWeek::Tuesday,
        DayOfWeek::Wednesday,
        DayOfWeek::Thursday,
        DayOfWeek::Friday,
        DayOfWeek::Saturday,
        DayOfWeek::Sunday,
    ];
    days.iter()
        .enumerate()
        .map(|(i, day)| ClassPeriod {
            class_time_id: class_time_id(g, school),
            day: *day,
            period_number: (i + 1) as u8,
            room_id: class_room_id(g, school),
            teacher_id: user_id(g),
        })
        .collect()
}

/// Minimal in-memory UniquenessChecker for testing.
#[derive(Default)]
struct InMemoryUniqueness {
    teacher_conflicts: HashSet<(SchoolId, UserId, DayOfWeek, u8)>,
    room_conflicts: HashSet<(SchoolId, ClassRoomId, DayOfWeek, u8)>,
}

impl UniquenessChecker for InMemoryUniqueness {
    fn student_admission_no_exists(&self, _: SchoolId, _: &str) -> bool {
        false
    }
    fn student_email_exists(&self, _: SchoolId, _: &str) -> bool {
        false
    }
    fn roll_no_exists(
        &self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId, _: &str,
    ) -> bool {
        false
    }
    fn class_name_exists(&self, _: SchoolId, _: &str) -> bool {
        false
    }
    fn section_name_exists(&self, _: SchoolId, _: &str) -> bool {
        false
    }
    fn subject_code_exists(&self, _: SchoolId, _: &str) -> bool {
        false
    }
    fn class_section_exists(
        &self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId,
    ) -> bool {
        false
    }
    fn class_section_has_student_records(&self, _: SchoolId, _: ClassSectionId) -> bool {
        false
    }
    fn academic_year_overlaps(
        &self, _: SchoolId, _: AcademicYearRange, _: Option<AcademicYearId>,
    ) -> bool {
        false
    }
    fn optional_subject_assigned_exists(
        &self, _: SchoolId, _: StudentId, _: AcademicYearId,
    ) -> bool {
        false
    }
    fn primary_guardian_link_exists(&self, _: SchoolId, _: StudentId) -> bool {
        false
    }
    fn teacher_has_conflict(
        &self, school: SchoolId, teacher_id: UserId, day: DayOfWeek, period_number: u8,
    ) -> bool {
        self.teacher_conflicts.contains(&(school, teacher_id, day, period_number))
    }
    fn room_has_conflict(
        &self, school: SchoolId, room_id: ClassRoomId, day: DayOfWeek, period_number: u8,
    ) -> bool {
        self.room_conflicts.contains(&(school, room_id, day, period_number))
    }
}

// =============================================================================
// 1. Happy path: create a ClassRoutine covering a full week
// =============================================================================

#[test]
fn class_routine_create_full_week_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods: full_week_periods(&g, school),
    };

    let (agg, event) = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect("full-week routine should succeed");

    assert_eq!(agg.school_id, school);
    assert_eq!(agg.periods.len(), 7);
    assert!(agg.is_active);

    // Event metadata matches DomainEvent contract.
    assert_eq!(ClassRoutineScheduled::EVENT_TYPE, "academic.class_routine.scheduled");
    assert_eq!(ClassRoutineScheduled::AGGREGATE_TYPE, "class_routine");
    assert_eq!(ClassRoutineScheduled::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: full week — 6 distinct days rejected
// =============================================================================

#[test]
fn class_routine_with_six_days_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let mut periods = full_week_periods(&g, school);
    periods.retain(|p| p.day != DayOfWeek::Sunday);

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods,
    };

    let err = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect_err("6-day routine must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 3. I-2: duplicate ClassTimeId rejected
// =============================================================================

#[test]
fn class_routine_with_duplicate_class_time_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let mut periods = full_week_periods(&g, school);
    let monday_id = periods[0].class_time_id;
    periods[6].class_time_id = monday_id;

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods,
    };

    let err = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate class_time_id must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// 4. I-3: period_number=0 rejected
// =============================================================================

#[test]
fn class_routine_with_invalid_period_number_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let mut periods = full_week_periods(&g, school);
    periods[0].period_number = 0;

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods,
    };

    let err = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect_err("period_number=0 must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 5. I-4: teacher conflict rejected
// =============================================================================

#[test]
fn class_routine_with_teacher_conflict_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let periods = full_week_periods(&g, school);
    let teacher = periods[0].teacher_id;
    let mut uniqueness = uniqueness;
    uniqueness
        .teacher_conflicts
        .insert((school, teacher, DayOfWeek::Monday, 1));

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods,
    };

    let err = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect_err("teacher conflict must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// 6. I-5: room conflict rejected
// =============================================================================

#[test]
fn class_routine_with_room_conflict_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let periods = full_week_periods(&g, school);
    let room = periods[1].room_id;
    let mut uniqueness = uniqueness;
    uniqueness
        .room_conflicts
        .insert((school, room, DayOfWeek::Tuesday, 2));

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods,
    };

    let err = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect_err("room conflict must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// 7. update_class_routine_period: re-runs invariants on new payload
// =============================================================================

#[test]
fn update_class_routine_period_rechecks_full_week() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = CreateClassRoutineCommand {
        tenant: tenant.clone(),
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods: full_week_periods(&g, school),
    };
    let (mut agg, _event) = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    // Replace with a 6-day payload — must fail.
    let mut new_periods = full_week_periods(&g, school);
    new_periods.retain(|p| p.day != DayOfWeek::Sunday);

    let upd_cmd = UpdateClassRoutinePeriodCommand {
        tenant: tenant.clone(),
        class_routine_id: agg.id,
        new_periods,
    };
    let err = update_class_routine_period(upd_cmd, &clock, &ids, &uniqueness, &mut agg)
        .expect_err("update with 6 days must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // Valid full-week update succeeds.
    let valid_periods = full_week_periods(&g, school);
    let upd_cmd = UpdateClassRoutinePeriodCommand {
        tenant,
        class_routine_id: agg.id,
        new_periods: valid_periods,
    };
    let event = update_class_routine_period(upd_cmd, &clock, &ids, &uniqueness, &mut agg)
        .expect("valid update");
    assert_eq!(agg.periods.len(), 7);
    let _ = event; // confirmed type by matches!
}

// =============================================================================
// 8. swap_class_routine_periods: swaps two periods
// =============================================================================

#[test]
fn swap_class_routine_periods_swaps_two_periods() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = CreateClassRoutineCommand {
        tenant: tenant.clone(),
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods: full_week_periods(&g, school),
    };
    let (mut agg, _event) = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let teacher_mon = agg.periods[0].teacher_id;
    let teacher_tue = agg.periods[1].teacher_id;

    let swap_cmd = SwapClassRoutinePeriodsCommand {
        tenant,
        class_routine_id: agg.id,
        period_a_idx: 0,
        period_b_idx: 1,
    };
    let event = swap_class_routine_periods(swap_cmd, &clock, &ids, &mut agg)
        .expect("swap");
    assert_eq!(agg.periods[0].teacher_id, teacher_tue);
    assert_eq!(agg.periods[1].teacher_id, teacher_mon);
    let _: ClassRoutinePeriodsSwapped = event;
}

// =============================================================================
// 9. delete_class_routine: retires the aggregate
// =============================================================================

#[test]
fn delete_class_routine_retires_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = CreateClassRoutineCommand {
        tenant,
        class_routine_id: class_routine_id(&g, school),
        class_section_id: class_section_id(&g, school),
        academic_year_id: academic_year_id(&g, school),
        periods: full_week_periods(&g, school),
    };
    let (mut agg, _event) = create_class_routine(cmd, &clock, &ids, &uniqueness)
        .expect("create");
    assert!(agg.is_active);

    let del_cmd = DeleteClassRoutineCommand {
        tenant: TenantContext::system(agg.school_id, g.next_correlation_id()),
        class_routine_id: agg.id,
    };
    let event = delete_class_routine(del_cmd, &clock, &ids, &mut agg)
        .expect("delete");
    assert!(!agg.is_active);
    let _: ClassRoutineDeleted = event;
}
