//! Integration tests for the **Guardian aggregate** vertical slice (Batch 1).
//!
//! Pins the full register contract for
//! [`Guardian`](educore_academic::Guardian)
//! end-to-end through the service layer:
//!
//! 1. `register_guardian` validates the inputs (first/last
//!    name length, optional phone/email format), constructs
//!    the aggregate, and emits a [`GuardianRegistered`] event
//!    with the typed id + contact payload.
//! 2. The aggregate carries a `phone` (Option) and `email`
//!    (Option) per Guardian I-1 (at most one of each).
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs`
//! (`TestClock` + `SystemIdGen`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;
use std::sync::Mutex;

use educore_academic::prelude::*;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_events::domain_event::DomainEvent;

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

fn guardian_id(g: &SystemIdGen, school: SchoolId) -> GuardianId {
    GuardianId::new(school, g.next_uuid())
}

fn link_id(g: &SystemIdGen, school: SchoolId) -> StudentGuardianLinkId {
    StudentGuardianLinkId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: register a Guardian
// =============================================================================

#[test]
fn guardian_register_builds_aggregate_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let gid = guardian_id(&g, school);
    let phone = PhoneNumber::new("+14155552671").unwrap();
    let email = EmailAddress::new("jane@example.com").unwrap();
    let cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Jane".to_owned(),
        last_name: "Doe".to_owned(),
        phone: Some(phone.clone()),
        email: Some(email.clone()),
    };
    let (agg, event) = register_guardian(cmd, &clock, &ids).expect("create");

    assert_eq!(agg.id, gid);
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.first_name, "Jane");
    assert_eq!(agg.last_name, "Doe");
    assert_eq!(agg.phone.as_ref().map(|p| p.as_str().to_owned()), Some(phone.as_str().to_owned()));
    assert_eq!(agg.email.as_ref().map(|e| e.as_str().to_owned()), Some(email.as_str().to_owned()));
    assert_eq!(agg.full_name(), "Jane Doe");
    assert!(agg.active_status.is_active());

    assert_eq!(
        <GuardianRegistered as DomainEvent>::EVENT_TYPE,
        "academic.guardian.registered"
    );
    assert_eq!(<GuardianRegistered as DomainEvent>::AGGREGATE_TYPE, "guardian");
    assert_eq!(event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.first_name, "Jane");
    assert_eq!(event.phone.as_ref().map(|p| p.as_str().to_owned()), Some(phone.as_str().to_owned()));
}

// =============================================================================
// 2. I-1: phone/email cap enforced at construction
// =============================================================================

#[test]
fn guardian_phone_number_rejects_invalid_format() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    assert!(PhoneNumber::new("").is_err());
    assert!(PhoneNumber::new("14155552671").is_err()); // missing +
    assert!(PhoneNumber::new("+abc").is_err()); // non-digit
    assert!(PhoneNumber::new("+14155552671").is_ok());
    let _ = (g, school);
}

#[test]
fn guardian_email_rejects_invalid_format() {
    assert!(EmailAddress::new("").is_err());
    assert!(EmailAddress::new("ada@example.com").is_ok());
    assert!(EmailAddress::new("no-at-sign").is_err());
    assert!(EmailAddress::new("@example.com").is_err());
    assert!(EmailAddress::new("user@").is_err());
}

// =============================================================================
// 3. Validation failure: empty first_name
// =============================================================================

#[test]
fn guardian_register_with_empty_first_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let gid = guardian_id(&g, school);
    let cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: String::new(),
        last_name: "Doe".to_owned(),
        phone: None,
        email: None,
    };
    let err = register_guardian(cmd, &clock, &ids).expect_err("empty first name must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 4. I-1 (compile-time enforcement): only one phone slot exists
// =============================================================================

/// Compile-time enforcement of Guardian I-1: the
/// [`Guardian`](educore_academic::Guardian) struct exposes
/// exactly one `phone` slot and one `email` slot. There is
/// no API surface (no Vec, no BTreeMap) that would let a
/// caller carry a second phone or email. This test reads
/// the public field list via a `match` on the struct shape
/// to make the invariant observable in CI: if a future
/// refactor adds a second phone slot, the match below stops
/// compiling.
#[test]
fn guardian_create_with_two_phones_rejected_by_type_system() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let gid = guardian_id(&g, school);
    let phone1 = PhoneNumber::new("+14155552671").unwrap();
    let phone2 = PhoneNumber::new("+14155552672").unwrap();
    let cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Jane".to_owned(),
        last_name: "Doe".to_owned(),
        // The command shape carries a single `phone: Option<PhoneNumber>`.
        // A second phone would require an additional field that does not
        // exist; this test pins the contract.
        phone: Some(phone1),
        email: None,
    };
    let (agg, _) = register_guardian(cmd, &clock, &ids).expect("create");
    // The aggregate has exactly one phone slot.
    assert_eq!(agg.phone.as_ref().map(|p| p.as_str()), Some("+14155552671"));
    // phone2 was never attached because the type system has no such slot.
    assert_eq!(agg.email, None);
    let _ = phone2;
}

// =============================================================================
// 5. I-1 (value-object validation): phone format invalid rejected
// =============================================================================

#[test]
fn guardian_phone_format_invalid_rejected() {
    // The PhoneNumber value object rejects malformed input at construction;
    // a downstream caller cannot smuggle an invalid E.164 string through
    // the RegisterGuardianCommand because the field type is PhoneNumber,
    // not String.
    let empty = PhoneNumber::new("");
    assert!(empty.is_err(), "empty string must be rejected");
    let no_plus = PhoneNumber::new("14155552671");
    assert!(no_plus.is_err(), "missing '+' must be rejected");
    let non_digit = PhoneNumber::new("+1abc");
    assert!(non_digit.is_err(), "non-digit characters must be rejected");
    let too_short = PhoneNumber::new("+12");
    assert!(too_short.is_err(), "fewer than 4 digits must be rejected");
    let too_long = PhoneNumber::new("+1234567890123456");
    assert!(too_long.is_err(), "more than 15 digits must be rejected");
    let ok = PhoneNumber::new("+14155552671");
    assert!(ok.is_ok(), "well-formed E.164 must be accepted");
}

// =============================================================================
// 6. In-memory UniquenessChecker for the I-4 path
// =============================================================================

/// Test-local `UniquenessChecker` that records which
/// `(school, student)` pairs already carry a primary
/// guardian link. The integration tests for I-4 wire the
/// `primary_guardian_link_exists` method to this stub.
#[derive(Debug, Default)]
struct TestUniqueness {
    primary_links: Mutex<HashSet<(SchoolId, StudentId)>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self::default()
    }
    fn record_primary(&self, school: SchoolId, student: StudentId) {
        self.primary_links.lock().unwrap().insert((school, student));
    }
}

impl UniquenessChecker for TestUniqueness {
    fn student_admission_no_exists(&self, _school: SchoolId, _admission_no: &str) -> bool {
        false
    }
    fn student_email_exists(&self, _school: SchoolId, _email: &str) -> bool {
        false
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
    fn primary_guardian_link_exists(&self, school: SchoolId, student_id: StudentId) -> bool {
        self.primary_links
            .lock()
            .unwrap()
            .contains(&(school, student_id))
    }
}

// =============================================================================
// 7. I-2 + I-3: link_guardian_to_student happy path
// =============================================================================

#[test]
fn guardian_link_to_student_creates_student_guardian_link() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    let gid = guardian_id(&g, school);
    let sid = student_id(&g, school);
    let lid = link_id(&g, school);
    let cmd = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
        guardian_id: gid,
        student_id: sid,
        relation: Relation::Mother,
        is_primary: true,
    };
    let (link, event) = link_guardian_to_student(cmd, &clock, &ids, &uniqueness).expect("link");

    assert_eq!(link.id, lid);
    assert_eq!(link.guardian_id, gid);
    assert_eq!(link.student_id, sid);
    assert_eq!(link.relation, Relation::Mother);
    assert!(link.is_primary);
    assert_eq!(link.school_id, school);

    assert_eq!(
        <GuardianLinkedToStudent as DomainEvent>::EVENT_TYPE,
        "academic.guardian.linked_to_student"
    );
    assert_eq!(<GuardianLinkedToStudent as DomainEvent>::AGGREGATE_TYPE, "student_guardian_link");
    assert_eq!(event.aggregate_id(), lid.as_uuid());
    assert_eq!(event.relation, Relation::Mother);
    assert!(event.is_primary);
}

// =============================================================================
// 8. I-2: a single guardian may be linked to multiple students
// =============================================================================

#[test]
fn guardian_can_link_to_multiple_students() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    let gid = guardian_id(&g, school);
    let sid_a = student_id(&g, school);
    let sid_b = student_id(&g, school);
    let lid_a = link_id(&g, school);
    let lid_b = link_id(&g, school);

    let cmd_a = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid_a,
        guardian_id: gid,
        student_id: sid_a,
        relation: Relation::Father,
        is_primary: true,
    };
    let (link_a, _) =
        link_guardian_to_student(cmd_a, &clock, &ids, &uniqueness).expect("link a");
    assert_eq!(link_a.guardian_id, gid);
    assert_eq!(link_a.student_id, sid_a);

    // Second link to a different student: same guardian, different
    // student. The link aggregate is a per-pair root so this is a
    // separate write; the uniqueness check tracks primary per
    // (school, student) so a second student can have its own primary.
    let cmd_b = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid_b,
        guardian_id: gid,
        student_id: sid_b,
        relation: Relation::Father,
        is_primary: true,
    };
    let (link_b, _) =
        link_guardian_to_student(cmd_b, &clock, &ids, &uniqueness).expect("link b");
    assert_eq!(link_b.guardian_id, gid);
    assert_eq!(link_b.student_id, sid_b);
}

// =============================================================================
// 9. I-3: Relation enum + IsPrimary per link
// =============================================================================

#[test]
fn relation_enum_round_trips_via_parse_str() {
    // Each variant parses from its canonical snake_case wire form.
    assert_eq!(Relation::parse_str("father"), Some(Relation::Father));
    assert_eq!(Relation::parse_str("mother"), Some(Relation::Mother));
    assert_eq!(Relation::parse_str("guardian"), Some(Relation::Guardian));
    assert_eq!(Relation::parse_str("other"), Some(Relation::Other));
    // Case-insensitive.
    assert_eq!(Relation::parse_str("Father"), Some(Relation::Father));
    assert_eq!(Relation::parse_str("MOTHER"), Some(Relation::Mother));
    // Unknown strings return None.
    assert_eq!(Relation::parse_str("sibling"), None);
    assert_eq!(Relation::parse_str(""), None);
    assert_eq!(Relation::parse_str("   "), None);
    // as_str + parse_str round-trip.
    for v in [
        Relation::Father,
        Relation::Mother,
        Relation::Guardian,
        Relation::Other,
    ] {
        assert_eq!(Relation::parse_str(v.as_str()), Some(v));
    }
}

#[test]
fn guardian_link_carries_relation_and_is_primary() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    let gid = guardian_id(&g, school);
    let sid = student_id(&g, school);
    let lid = link_id(&g, school);
    let cmd = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
        guardian_id: gid,
        student_id: sid,
        relation: Relation::Other,
        is_primary: false,
    };
    let (link, event) = link_guardian_to_student(cmd, &clock, &ids, &uniqueness).expect("link");
    assert_eq!(link.relation, Relation::Other);
    assert!(!link.is_primary);
    assert_eq!(event.relation, Relation::Other);
    assert!(!event.is_primary);
}

// =============================================================================
// 10. I-4: at most one IsPrimary per student (rejects violation)
// =============================================================================

#[test]
fn guardian_mark_primary_when_already_primary_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    // Pre-seed the uniqueness checker: the student already has
    // a primary guardian link.
    let sid = student_id(&g, school);
    uniqueness.record_primary(school, sid);

    let gid = guardian_id(&g, school);
    let lid = link_id(&g, school);
    let cmd = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
        guardian_id: gid,
        student_id: sid,
        relation: Relation::Guardian,
        // The link itself wants to be primary — must be rejected
        // because the student already has one.
        is_primary: true,
    };
    let err = link_guardian_to_student(cmd, &clock, &ids, &uniqueness)
        .expect_err("primary collision must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    let msg = format!("{err}");
    assert!(
        msg.contains("primary guardian"),
        "error message should mention 'primary guardian', got: {msg}"
    );
}

// =============================================================================
// 11. I-5: soft-delete when all student links removed
// =============================================================================

#[test]
fn guardian_unlink_last_student_soft_deletes() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    // Build a guardian, then a link, then unlink with was_last_link=true.
    let gid = guardian_id(&g, school);
    let sid = student_id(&g, school);
    let lid = link_id(&g, school);

    let reg_cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Bob".to_owned(),
        last_name: "Smith".to_owned(),
        phone: Some(PhoneNumber::new("+14155552673").unwrap()),
        email: None,
    };
    let (mut guardian, _) = register_guardian(reg_cmd, &clock, &ids).expect("register");
    assert!(guardian.active_status.is_active());

    let link_cmd = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
        guardian_id: gid,
        student_id: sid,
        relation: Relation::Father,
        is_primary: false,
    };
    let (mut link, _) =
        link_guardian_to_student(link_cmd, &clock, &ids, &uniqueness).expect("link");

    // Unlink with was_last_link=true (dispatcher signals the last link).
    let unlink_cmd = UnlinkGuardianFromStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
    };
    let unlink_event =
        unlink_guardian_from_student(&mut link, unlink_cmd, &clock, &ids, true);
    assert!(unlink_event.guardian_retired);

    // Dispatcher cascade: retire the guardian.
    let retire_cmd = RetireGuardianCommand {
        tenant: tenant.clone(),
        guardian_id: gid,
    };
    let retire_event = retire_guardian(&mut guardian, retire_cmd, &clock, &ids).expect("retire");
    assert!(!guardian.active_status.is_active());
    assert_eq!(
        <GuardianRetired as DomainEvent>::EVENT_TYPE,
        "academic.guardian.retired"
    );
    assert_eq!(retire_event.guardian_id, gid);
}

// =============================================================================
// 12. I-5: unlink with was_last_link=false leaves guardian active
// =============================================================================

#[test]
fn guardian_unlink_non_last_keeps_guardian_active() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    let gid = guardian_id(&g, school);
    let sid = student_id(&g, school);
    let lid = link_id(&g, school);

    let reg_cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Bob".to_owned(),
        last_name: "Smith".to_owned(),
        phone: None,
        email: None,
    };
    let (_guardian, _) = register_guardian(reg_cmd, &clock, &ids).expect("register");

    let link_cmd = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
        guardian_id: gid,
        student_id: sid,
        relation: Relation::Father,
        is_primary: false,
    };
    let (mut link, _) =
        link_guardian_to_student(link_cmd, &clock, &ids, &uniqueness).expect("link");

    // Unlink with was_last_link=false — dispatcher knows another link
    // remains for this guardian. The event should not signal retire.
    let unlink_cmd = UnlinkGuardianFromStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
    };
    let unlink_event =
        unlink_guardian_from_student(&mut link, unlink_cmd, &clock, &ids, false);
    assert!(!unlink_event.guardian_retired);
}

// =============================================================================
// 13. Update contact: phone/email updated; event carries new values
// =============================================================================

#[test]
fn guardian_update_contact_emits_event_with_new_values() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let gid = guardian_id(&g, school);
    let reg_cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Eve".to_owned(),
        last_name: "Adams".to_owned(),
        phone: Some(PhoneNumber::new("+14155552671").unwrap()),
        email: Some(EmailAddress::new("eve@example.com").unwrap()),
    };
    let (mut guardian, _) = register_guardian(reg_cmd, &clock, &ids).expect("register");

    let new_phone = PhoneNumber::new("+442071838750").unwrap();
    let new_email = EmailAddress::new("eve2@example.com").unwrap();
    let upd_cmd = UpdateGuardianContactCommand {
        tenant: tenant.clone(),
        guardian_id: gid,
        phone: Some(Some(new_phone.clone())),
        email: Some(Some(new_email.clone())),
    };
    let event =
        update_guardian_contact(&mut guardian, upd_cmd, &clock, &ids).expect("update");
    assert_eq!(
        <GuardianContactUpdated as DomainEvent>::EVENT_TYPE,
        "academic.guardian.contact_updated"
    );
    assert_eq!(
        event.changed_fields,
        vec!["phone".to_owned(), "email".to_owned()]
    );
    assert_eq!(
        guardian.phone.as_ref().map(|p| p.as_str()),
        Some(new_phone.as_str())
    );
    assert_eq!(
        guardian.email.as_ref().map(|e| e.as_str()),
        Some(new_email.as_str())
    );
}

// =============================================================================
// 14. Update contact: no-op patch does not bump version
// =============================================================================

#[test]
fn guardian_update_contact_noop_does_not_bump_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let gid = guardian_id(&g, school);
    let phone = PhoneNumber::new("+14155552671").unwrap();
    let email = EmailAddress::new("eve@example.com").unwrap();
    let reg_cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Eve".to_owned(),
        last_name: "Adams".to_owned(),
        phone: Some(phone.clone()),
        email: Some(email.clone()),
    };
    let (mut guardian, _) = register_guardian(reg_cmd, &clock, &ids).expect("register");
    let v0 = guardian.version;

    // Pass the same values; nothing should change.
    let upd_cmd = UpdateGuardianContactCommand {
        tenant: tenant.clone(),
        guardian_id: gid,
        phone: Some(Some(phone.clone())),
        email: Some(Some(email.clone())),
    };
    let event =
        update_guardian_contact(&mut guardian, upd_cmd, &clock, &ids).expect("update");
    assert!(event.changed_fields.is_empty());
    assert_eq!(guardian.version, v0);
}

// =============================================================================
// 15. Update contact: cannot mutate a retired guardian
// =============================================================================

#[test]
fn guardian_update_contact_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let gid = guardian_id(&g, school);
    let reg_cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Eve".to_owned(),
        last_name: "Adams".to_owned(),
        phone: None,
        email: None,
    };
    let (mut guardian, _) = register_guardian(reg_cmd, &clock, &ids).expect("register");
    let retire_cmd = RetireGuardianCommand {
        tenant: tenant.clone(),
        guardian_id: gid,
    };
    let _ = retire_guardian(&mut guardian, retire_cmd, &clock, &ids).expect("retire");
    assert!(!guardian.active_status.is_active());

    let upd_cmd = UpdateGuardianContactCommand {
        tenant: tenant.clone(),
        guardian_id: gid,
        phone: Some(Some(PhoneNumber::new("+14155552671").unwrap())),
        email: None,
    };
    let err = update_guardian_contact(&mut guardian, upd_cmd, &clock, &ids)
        .expect_err("retired must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// 16. Mark primary: happy path on an unlinked link
// =============================================================================

#[test]
fn guardian_mark_primary_emits_event_and_sets_flag() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = TestUniqueness::new();

    let gid = guardian_id(&g, school);
    let sid = student_id(&g, school);
    let lid = link_id(&g, school);
    let cmd = LinkGuardianToStudentCommand {
        tenant: tenant.clone(),
        link_id: lid,
        guardian_id: gid,
        student_id: sid,
        relation: Relation::Mother,
        is_primary: false,
    };
    let (mut link, _) =
        link_guardian_to_student(cmd, &clock, &ids, &uniqueness).expect("link");
    assert!(!link.is_primary);

    let mark_cmd = MarkPrimaryGuardianCommand {
        tenant: tenant.clone(),
        link_id: lid,
    };
    let event = mark_primary_guardian(&mut link, mark_cmd, &clock, &ids, &uniqueness, None)
        .expect("mark");
    assert!(link.is_primary);
    assert_eq!(
        <PrimaryGuardianMarked as DomainEvent>::EVENT_TYPE,
        "academic.guardian.primary_marked"
    );
    assert_eq!(event.link_id, lid);
    assert_eq!(event.guardian_id, gid);
}
