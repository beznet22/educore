//! End-to-end integration tests for the platform crate.
//!
//! The tests exercise the full command → aggregate → typed
//! event → envelope flow against the in-memory
//! [`UniquenessChecker`] and clock/id generator ports. They
//! are intentionally storage-agnostic: the engine's command
//! dispatcher is responsible for persisting the aggregate and
//! publishing the event; the unit under test here is the
//! service layer's contract (i.e. the typed event metadata
//! and the aggregate mutation).
//!
//! The six tests cover the exact assertions listed in the
//! Phase 2 Workstream C prompt:
//! 1. `school_create_emits_school_created_event` — happy
//!    path: create a school, wrap the event in an envelope,
//!    assert the envelope's `event_type` is the namespaced
//!    `platform.school.created`.
//! 2. `user_register_emits_user_registered_event` — happy
//!    path: register a user, wrap, assert the envelope's
//!    `event_type` is `platform.user.registered`.
//! 3. `school_uniqueness_constraint_enforced` — second
//!    `create_school` with the same `school_code` returns
//!    `Err(DomainError::Conflict)`.
//! 4. `user_email_uniqueness_within_school` — second
//!    `register_user` with the same email in the same school
//!    returns `Err(DomainError::Conflict)`.
//! 5. `update_school_increments_version` — two
//!    `update_school` calls on a freshly-created school bring
//!    the aggregate's `version` from 1 to 3 (initial + 2
//!    updates).
//! 6. `deactivate_user_sets_active_status_retired` — after
//!    `deactivate_user`, `user.active_status ==
//!    ActiveStatus::Retired`.

// The integration tests intentionally use `unwrap` to
// surface failures as test panics with line numbers; the
// workspace lints deny `unwrap` in production code only.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Mutex;

use educore_core::clock::{Clock, DeterministicIdGen, IdGenerator, SystemIdGen, TestClock};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId, PLATFORM_SCHOOL_ID};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;

use educore_platform::commands::{
    CreateSchoolCommand, DeactivateUserCommand, RegisterUserCommand, UniquenessChecker,
    UpdateSchoolCommand,
};
use educore_platform::events::{SchoolCreated, SchoolUpdated, UserDeactivated, UserRegistered};
use educore_platform::prelude::{
    create_school, deactivate_user, register_user, update_school, EmailAddress, HashedPassword,
    School, User,
};
use educore_platform::value_objects::{RoleId, SchoolStatus, UserStatus};

/// In-memory [`UniquenessChecker`] for the integration tests.
#[derive(Default)]
struct InMemoryUniqueness {
    codes: Mutex<Vec<String>>,
    domains: Mutex<Vec<String>>,
    emails: Mutex<Vec<(SchoolId, String)>>,
    usernames: Mutex<Vec<(SchoolId, String)>>,
}

impl InMemoryUniqueness {
    fn new() -> Self {
        Self::default()
    }

    fn record_school(&self, code: &str, domain: Option<&str>) {
        self.codes.lock().unwrap().push(code.to_owned());
        if let Some(d) = domain {
            self.domains.lock().unwrap().push(d.to_owned());
        }
    }

    fn record_user(&self, school: SchoolId, email: &str, username: &str) {
        self.emails
            .lock()
            .unwrap()
            .push((school, email.to_lowercase()));
        self.usernames
            .lock()
            .unwrap()
            .push((school, username.to_owned()));
    }
}

impl UniquenessChecker for InMemoryUniqueness {
    fn school_code_exists(&self, code: &str) -> bool {
        self.codes.lock().unwrap().iter().any(|c| c == code)
    }
    fn school_domain_exists(&self, domain: &str) -> bool {
        self.domains.lock().unwrap().iter().any(|d| d == domain)
    }
    fn user_email_exists(&self, school: SchoolId, email: &str) -> bool {
        let e = email.to_lowercase();
        self.emails
            .lock()
            .unwrap()
            .iter()
            .any(|(s, m)| *s == school && m == &e)
    }
    fn user_username_exists(&self, school: SchoolId, username: &str) -> bool {
        self.usernames
            .lock()
            .unwrap()
            .iter()
            .any(|(s, u)| *s == school && u == username)
    }
}

fn etag() -> Etag {
    Etag::new("0123456789abcdef0123456789abcdef").unwrap()
}

#[allow(dead_code)]
fn _exercise_etags() -> Etag {
    // The default etag is the 32-zero placeholder; the storage
    // adapter will overwrite this on the first successful
    // insert. The integration tests below exercise the
    // service-layer path; the placeholder is fine for the
    // round-trip.
    etag()
}

fn system_ctx(school: SchoolId) -> TenantContext {
    let g = SystemIdGen;
    TenantContext::system(school, g.next_correlation_id())
}

#[test]
fn school_create_emits_school_created_event() {
    let g = DeterministicIdGen::starting_at(0);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);
    let cmd = CreateSchoolCommand::new(
        ctx_local.clone(),
        school_id,
        "Ada".to_owned(),
        "ADA".to_owned(),
    );
    let (school, event): (School, SchoolCreated) = create_school(cmd, &clock, &g, &u).unwrap();

    assert_eq!(school.id, school_id);
    assert_eq!(event.school_id(), school_id);
    assert_eq!(event.name, "Ada");
    assert_eq!(event.school_code, "ADA");
    assert_eq!(event.occurred_at, clock.now());

    let envelope: EventEnvelope = event.into_envelope(&ctx_local);
    assert_eq!(envelope.event_type, "platform.school.created");
    assert_eq!(envelope.aggregate_type, "school");
    assert_eq!(envelope.school_id, school_id);
    assert_eq!(envelope.aggregate_id, school_id.as_uuid());
    assert_eq!(envelope.schema_version, 1);
}

#[test]
fn user_register_emits_user_registered_event() {
    let g = DeterministicIdGen::starting_at(100);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);
    let user_id = g.next_user_id();
    let cmd = RegisterUserCommand::new(
        ctx_local.clone(),
        user_id,
        school_id,
        EmailAddress::new("ada@example.com").unwrap(),
        "ada".to_owned(),
        "Ada".to_owned(),
        HashedPassword::from_hash("$argon2id$dummy"),
    );
    let (user, event): (User, UserRegistered) = register_user(cmd, &clock, &g, &u).unwrap();

    assert_eq!(user.id, user_id);
    assert_eq!(user.school_id, school_id);
    assert_eq!(user.email.as_str(), "ada@example.com");
    assert!(user.status.can_authenticate());
    assert_eq!(event.user_id, user_id);
    assert_eq!(event.school_id(), school_id);
    assert_eq!(event.aggregate_id(), user_id.as_uuid());

    let envelope: EventEnvelope = event.into_envelope(&ctx_local);
    assert_eq!(envelope.event_type, "platform.user.registered");
    assert_eq!(envelope.aggregate_type, "user");
    assert_eq!(envelope.school_id, school_id);
}

#[test]
fn school_uniqueness_constraint_enforced() {
    let g = DeterministicIdGen::starting_at(200);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id_a = g.next_school_id();
    let ctx_a = system_ctx(school_id_a);
    let cmd_a = CreateSchoolCommand::new(
        ctx_a.clone(),
        school_id_a,
        "Ada".to_owned(),
        "ADA".to_owned(),
    );
    let (school_a, _) = create_school(cmd_a, &clock, &g, &u).unwrap();
    assert_eq!(school_a.school_code, "ADA");

    // Record the school so the uniqueness check sees it.
    u.record_school("ADA", None);

    let school_id_b = g.next_school_id();
    let ctx_b = system_ctx(school_id_b);
    let cmd_b =
        CreateSchoolCommand::new(ctx_b, school_id_b, "Babbage".to_owned(), "ADA".to_owned());
    let err = create_school(cmd_b, &clock, &g, &u).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

#[test]
fn user_email_uniqueness_within_school() {
    let g = DeterministicIdGen::starting_at(300);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);

    let user_a = g.next_user_id();
    let cmd_a = RegisterUserCommand::new(
        ctx_local.clone(),
        user_a,
        school_id,
        EmailAddress::new("ada@example.com").unwrap(),
        "ada".to_owned(),
        "Ada".to_owned(),
        HashedPassword::from_hash("$argon2id$dummy"),
    );
    let (_user_a, _) = register_user(cmd_a, &clock, &g, &u).unwrap();

    // Record the user so the uniqueness check sees it.
    u.record_user(school_id, "ada@example.com", "ada");

    let user_b = g.next_user_id();
    let cmd_b = RegisterUserCommand::new(
        ctx_local,
        user_b,
        school_id,
        EmailAddress::new("Ada@Example.COM").unwrap(),
        "ada2".to_owned(),
        "Ada Lovelace".to_owned(),
        HashedPassword::from_hash("$argon2id$dummy"),
    );
    let err = register_user(cmd_b, &clock, &g, &u).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

#[test]
fn update_school_increments_version() {
    let g = DeterministicIdGen::starting_at(400);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);
    let cmd = CreateSchoolCommand::new(
        ctx_local.clone(),
        school_id,
        "Ada".to_owned(),
        "ADA".to_owned(),
    );
    let (mut school, _) = create_school(cmd, &clock, &g, &u).unwrap();
    assert_eq!(school.version, Version::initial());

    let upd = UpdateSchoolCommand {
        tenant: ctx_local.clone(),
        school_id,
        name: Some("Ada Lovelace".to_owned()),
        domain: None,
        package_id: None,
    };
    let event1: SchoolUpdated = update_school(&ctx_local, &mut school, upd, &clock, &u).unwrap();
    assert_eq!(school.version, Version::initial().next());
    assert_eq!(event1.changed_fields, vec!["name".to_owned()]);

    let upd2 = UpdateSchoolCommand {
        tenant: ctx_local.clone(),
        school_id,
        name: Some("Ada, Countess of Lovelace".to_owned()),
        domain: None,
        package_id: None,
    };
    let event2: SchoolUpdated = update_school(&ctx_local, &mut school, upd2, &clock, &u).unwrap();
    assert_eq!(school.version, Version::initial().next().next());
    assert_eq!(event2.changed_fields, vec!["name".to_owned()]);
}

#[test]
fn deactivate_user_sets_active_status_retired() {
    let g = DeterministicIdGen::starting_at(500);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);
    let user_id = g.next_user_id();
    let cmd = RegisterUserCommand::new(
        ctx_local.clone(),
        user_id,
        school_id,
        EmailAddress::new("ada@example.com").unwrap(),
        "ada".to_owned(),
        "Ada".to_owned(),
        HashedPassword::from_hash("$argon2id$dummy"),
    );
    let (mut user, _) = register_user(cmd, &clock, &g, &u).unwrap();
    assert!(user.active_status.is_active());
    assert_eq!(user.status, UserStatus::Active);

    let deact = DeactivateUserCommand::new(ctx_local.clone(), user_id, "resigned");
    let event: UserDeactivated = deactivate_user(&ctx_local, &mut user, deact, &clock).unwrap();
    assert!(user.active_status.is_retired());
    assert_eq!(user.status, UserStatus::Inactive);
    assert_eq!(event.user_id, user_id);
    assert_eq!(event.new_status, UserStatus::Inactive);
}

/// Bonus test: assert the typed event's `into_envelope` helper
/// propagates the correlation id from the `TenantContext`.
#[test]
fn envelope_propagates_correlation_id() {
    let g = DeterministicIdGen::starting_at(600);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let corr = g.next_correlation_id();
    let actor = g.next_user_id();
    let ctx_local = TenantContext::for_user(school_id, actor, corr, UserType::SchoolAdmin);
    let cmd = CreateSchoolCommand::new(
        ctx_local.clone(),
        school_id,
        "Ada".to_owned(),
        "ADA".to_owned(),
    );
    let (_school, event) = create_school(cmd, &clock, &g, &u).unwrap();
    let envelope: EventEnvelope = event.into_envelope(&ctx_local);
    assert_eq!(envelope.correlation_id, corr);
    assert_eq!(envelope.actor_id, actor);
}

/// Bonus test: assert `School::status` starts as `Pending` and
/// the `EventEnvelope::event_id` round-trips through the
/// `into_envelope` helper.
#[test]
fn school_starts_pending_and_event_id_round_trips() {
    let g = DeterministicIdGen::starting_at(700);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);
    let cmd = CreateSchoolCommand::new(
        ctx_local.clone(),
        school_id,
        "Ada".to_owned(),
        "ADA".to_owned(),
    );
    let (school, event) = create_school(cmd, &clock, &g, &u).unwrap();
    assert_eq!(school.status, SchoolStatus::Pending);
    let event_id = event.event_id;
    let envelope = event.into_envelope(&ctx_local);
    assert_eq!(envelope.event_id, EventId::from_uuid(event_id.0));
}

/// Bonus test: assert the `User`'s `version` increments on
/// `deactivate_user`.
#[test]
fn deactivate_user_increments_user_version() {
    let g = DeterministicIdGen::starting_at(800);
    let clock = TestClock::new();
    let u = InMemoryUniqueness::new();
    let school_id = g.next_school_id();
    let ctx_local = system_ctx(school_id);
    let user_id = g.next_user_id();
    let cmd = RegisterUserCommand::new(
        ctx_local.clone(),
        user_id,
        school_id,
        EmailAddress::new("ada@example.com").unwrap(),
        "ada".to_owned(),
        "Ada".to_owned(),
        HashedPassword::from_hash("$argon2id$dummy"),
    );
    let (mut user, _) = register_user(cmd, &clock, &g, &u).unwrap();
    let v0 = user.version;
    let deact = DeactivateUserCommand::new(ctx_local.clone(), user_id, "resigned");
    deactivate_user(&ctx_local, &mut user, deact, &clock).unwrap();
    assert_eq!(user.version, v0.next());
}

/// Touch a few `Timestamp` / `Version` / `ActiveStatus`
/// constants so the unused-imports lint does not fire on
/// edge builds.
#[test]
fn ensure_extras_compile() {
    let _: Timestamp = Timestamp::epoch();
    let _: Version = Version::initial();
    let _: ActiveStatus = ActiveStatus::Active;
    let _: SchoolId = PLATFORM_SCHOOL_ID;
    let _: CorrelationId = CorrelationId(uuid::Uuid::nil());
    // The `RoleId` value object is not exercised by the
    // default registration flow but is re-exported in the
    // prelude. Touch the type so the linter does not flag
    // the import.
    let _: RoleId = RoleId::new(PLATFORM_SCHOOL_ID, uuid::Uuid::nil());
    let _id: UserId = UserId::from_uuid(uuid::Uuid::now_v7());
}
