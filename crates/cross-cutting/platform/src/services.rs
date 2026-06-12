//! Platform-domain factory functions.
//!
//! The services module is the **only** place the engine
//! mutates a platform aggregate and emits its typed event.
//! Every function is **pure**: it takes the command, the
//! current `TenantContext`, a `Clock`, an `IdGenerator`, and
//! (for create / register flows) a
//! [`UniquenessChecker`](crate::commands::UniquenessChecker),
//! and returns the mutated aggregate plus the typed event.
//! The dispatcher (in the engine's core) is responsible for
//! persisting the aggregate and publishing the event under a
//! single transaction.
//!
//! Phase 2 ships:
//! - [`create_school`] → `(School, SchoolCreated)`
//! - [`update_school`] → `SchoolUpdated` (mutates the `School`
//!   in place; returns the event)
//! - [`deactivate_school`] → `SchoolDeactivated` (mutates the
//!   `School` in place; returns the event)
//! - [`register_user`] → `(User, UserRegistered)`
//! - [`update_user`] → `UserUpdated` (mutates the `User` in
//!   place; returns the event)
//! - [`deactivate_user`] → `UserDeactivated` (mutates the
//!   `User` in place; returns the event)

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::Identifier;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::ActiveStatus;

use crate::aggregate::{School, User};
use crate::commands::{
    validate_display_name, validate_reason, validate_school_code, validate_school_name,
    validate_username, CreateSchoolCommand, DeactivateSchoolCommand, DeactivateUserCommand,
    RegisterUserCommand, UniquenessChecker, UpdateSchoolCommand, UpdateUserCommand,
};
use crate::events::{
    SchoolCreated, SchoolDeactivated, SchoolUpdated, UserDeactivated, UserRegistered, UserUpdated,
};
use crate::value_objects::SchoolStatus;

/// Create a new [`School`] and emit a [`SchoolCreated`] event.
///
/// Returns the new `School` and the typed event. The caller
/// (the engine's command dispatcher) is responsible for
/// persisting the aggregate and publishing the event under a
/// single transaction.
///
/// # Errors
///
/// - `Validation` if any of `cmd.name`, `cmd.school_code`, or
///   `cmd.domain` (when present) fails structural validation.
/// - `Conflict` if `cmd.school_code` (or `cmd.domain`, when
///   present) is already taken per the `uniqueness` checker.
pub fn create_school<C, G>(
    cmd: CreateSchoolCommand,
    clock: &C,
    _ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(School, SchoolCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let CreateSchoolCommand {
        tenant,
        school_id,
        name,
        school_code,
        domain,
        package_id,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_school_name(&name)?;
    validate_school_code(&school_code)?;
    if uniqueness.school_code_exists(&school_code) {
        return Err(DomainError::Conflict(format!(
            "school code {school_code:?} is already taken"
        )));
    }
    if let Some(d) = domain.as_deref() {
        if uniqueness.school_domain_exists(d) {
            return Err(DomainError::Conflict(format!(
                "school domain {d:?} is already taken"
            )));
        }
    }

    let mut school = School::fresh(
        school_id,
        name.clone(),
        school_code.clone(),
        domain.clone(),
        package_id,
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    let event_id = _ids.next_event_id();
    school.last_event_id = Some(event_id);
    school.correlation_id = ctx.correlation_id;

    let event = SchoolCreated::new(
        school_id,
        name,
        school_code,
        domain,
        SchoolStatus::Pending,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok((school, event))
}

/// Update a [`School`] in place and emit a [`SchoolUpdated`]
/// event.
///
/// Returns the typed event. The caller is responsible for
/// persisting the mutated aggregate and publishing the event.
///
/// # Errors
///
/// - `Validation` if any of the supplied fields fails
///   structural validation.
/// - `Conflict` if the new `domain` (when supplied) is
///   already taken by another school.
pub fn update_school<C>(
    ctx: &TenantContext,
    school: &mut School,
    cmd: UpdateSchoolCommand,
    clock: &C,
    uniqueness: &dyn UniquenessChecker,
) -> Result<SchoolUpdated>
where
    C: Clock + ?Sized,
{
    let UpdateSchoolCommand {
        tenant,
        school_id,
        name,
        domain,
        package_id,
    } = cmd;
    debug_assert_eq!(
        tenant.school_id, school_id,
        "command's school_id must match the aggregate's school_id"
    );
    let _ = ctx;
    let now = clock.now();
    let mut changed = Vec::new();

    if let Some(new_name) = name {
        validate_school_name(&new_name)?;
        if new_name != school.name {
            changed.push("name".to_owned());
            school.name = new_name;
        }
    }
    if let Some(new_domain) = domain {
        if let Some(d) = new_domain.as_deref() {
            if uniqueness.school_domain_exists(d) && school.domain.as_deref() != Some(d) {
                return Err(DomainError::Conflict(format!(
                    "school domain {d:?} is already taken"
                )));
            }
        }
        if new_domain != school.domain {
            changed.push("domain".to_owned());
            school.domain = new_domain;
        }
    }
    if let Some(new_pkg) = package_id {
        if new_pkg != school.package_id {
            changed.push("package_id".to_owned());
            school.package_id = new_pkg;
        }
    }

    if !changed.is_empty() {
        school.updated_at = now;
        school.updated_by = ctx.actor_id;
        school.version = school.version.next();
    }

    // The event id is not known until the storage adapter
    // commits; we mint a placeholder event id here for the
    // envelope (the bus port stamps its own event id at
    // publish time, so this is informational only).
    let event_id = {
        let mut bytes = [0u8; 16];
        let s_id = school.id.as_uuid();
        bytes.copy_from_slice(s_id.as_bytes());
        educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7())
    };
    school.last_event_id = Some(event_id);

    let event = SchoolUpdated::new(
        school_id,
        changed,
        school.name.clone().into(),
        school.domain.clone(),
        school.package_id.map(|p| p.as_uuid()),
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok(event)
}

/// Deactivate a [`School`] in place and emit a
/// [`SchoolDeactivated`] event.
///
/// Sets `school.status = new_status` and
/// `school.active_status = Retired`. The caller is responsible
/// for persisting the mutated aggregate and publishing the
/// event.
///
/// # Errors
///
/// - `Validation` if `cmd.reason` is empty or too long.
pub fn deactivate_school<C>(
    ctx: &TenantContext,
    school: &mut School,
    cmd: DeactivateSchoolCommand,
    clock: &C,
) -> Result<SchoolDeactivated>
where
    C: Clock + ?Sized,
{
    let DeactivateSchoolCommand {
        tenant,
        school_id,
        reason,
        new_status,
    } = cmd;
    debug_assert_eq!(tenant.school_id, school_id);
    let _ = ctx;
    validate_reason(&reason)?;
    let now = clock.now();
    school.status = new_status;
    school.active_status = ActiveStatus::Retired;
    school.updated_at = now;
    school.updated_by = ctx.actor_id;
    school.version = school.version.next();
    let event_id = educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7());
    school.last_event_id = Some(event_id);
    let event = SchoolDeactivated::new(
        school_id,
        reason,
        new_status,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok(event)
}

/// Register a new [`User`] and emit a [`UserRegistered`]
/// event.
///
/// Returns the new `User` and the typed event. The caller is
/// responsible for persisting the aggregate and publishing
/// the event under a single transaction.
///
/// # Errors
///
/// - `Validation` if any of the command's string fields
///   fails structural validation.
/// - `Conflict` if `cmd.email` or `cmd.username` is already
///   taken within `cmd.school_id` per the `uniqueness`
///   checker.
pub fn register_user<C, G>(
    cmd: RegisterUserCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn UniquenessChecker,
) -> Result<(User, UserRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let RegisterUserCommand {
        tenant,
        user_id,
        school_id,
        email,
        username,
        display_name,
        phone_number,
        usertype,
        role_ids,
        password_hash,
    } = cmd;
    let ctx = tenant;
    let now = clock.now();
    validate_username(&username)?;
    validate_display_name(&display_name)?;
    if uniqueness.user_email_exists(school_id, email.as_str()) {
        return Err(DomainError::Conflict(format!(
            "email {:?} is already in use within the school",
            email.as_str()
        )));
    }
    if uniqueness.user_username_exists(school_id, &username) {
        return Err(DomainError::Conflict(format!(
            "username {username:?} is already in use within the school"
        )));
    }
    let role_inner: Vec<uuid::Uuid> = role_ids.iter().map(|r| r.1).collect();

    let mut user = User::fresh(
        user_id,
        school_id,
        email.clone(),
        username,
        display_name,
        phone_number,
        usertype,
        role_ids,
        password_hash,
        ctx.actor_id,
        ctx.actor_id,
        now,
        ctx.correlation_id,
    );
    let event_id = ids.next_event_id();
    user.last_event_id = Some(event_id);
    user.correlation_id = ctx.correlation_id;

    let event = UserRegistered::new(
        user_id,
        school_id,
        email,
        user.username.clone(),
        user.display_name.clone(),
        user.usertype,
        role_inner,
        user.status,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok((user, event))
}

/// Update a [`User`] in place and emit a [`UserUpdated`]
/// event.
///
/// Returns the typed event. The caller is responsible for
/// persisting the mutated aggregate and publishing the event.
///
/// # Errors
///
/// - `Validation` if any of the supplied fields fails
///   structural validation.
/// - `Conflict` if the new `email` (when supplied) is
///   already taken within the school.
pub fn update_user<C>(
    ctx: &TenantContext,
    user: &mut User,
    cmd: UpdateUserCommand,
    clock: &C,
    uniqueness: &dyn UniquenessChecker,
) -> Result<UserUpdated>
where
    C: Clock + ?Sized,
{
    let UpdateUserCommand {
        tenant,
        user_id,
        email,
        display_name,
        phone_number,
    } = cmd;
    debug_assert_eq!(tenant.school_id, user.school_id);
    let _ = ctx;
    let now = clock.now();
    let mut changed = Vec::new();

    // Snapshot the pre-mutation values so the event can carry
    // "what changed" without aliasing the post-mutation state.
    let email_at_call = user.email.clone();
    let display_name_at_call = user.display_name.clone();
    let phone_at_call = user.phone_number.clone();

    if let Some(new_email) = email {
        if uniqueness.user_email_exists(user.school_id, new_email.as_str())
            && new_email != user.email
        {
            return Err(DomainError::Conflict(format!(
                "email {:?} is already in use within the school",
                new_email.as_str()
            )));
        }
        if new_email != user.email {
            changed.push("email".to_owned());
            user.email = new_email;
        }
    }
    if let Some(new_name) = display_name {
        validate_display_name(&new_name)?;
        if new_name != user.display_name {
            changed.push("display_name".to_owned());
            user.display_name = new_name;
        }
    }
    if let Some(new_phone) = phone_number {
        if new_phone != user.phone_number {
            changed.push("phone_number".to_owned());
            user.phone_number = new_phone;
        }
    }

    if !changed.is_empty() {
        user.updated_at = now;
        user.updated_by = ctx.actor_id;
        user.version = user.version.next();
    }
    let event_id = educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7());
    user.last_event_id = Some(event_id);

    let email_changed = changed.iter().any(|f| f == "email");
    let display_name_changed = changed.iter().any(|f| f == "display_name");
    let phone_changed = changed.iter().any(|f| f == "phone_number");
    let _ = (email_at_call, display_name_at_call, phone_at_call);
    let event = UserUpdated::new(
        user_id,
        user.school_id,
        changed,
        if email_changed {
            Some(user.email.clone())
        } else {
            None
        },
        if display_name_changed {
            Some(user.display_name.clone())
        } else {
            None
        },
        if phone_changed {
            user.phone_number.as_ref().map(|p| p.as_str().to_owned())
        } else {
            None
        },
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok(event)
}

/// Deactivate a [`User`] in place and emit a
/// [`UserDeactivated`] event.
///
/// Sets `user.status = new_status` and
/// `user.active_status = Retired`. The caller is responsible
/// for persisting the mutated aggregate and publishing the
/// event.
///
/// # Errors
///
/// - `Validation` if `cmd.reason` is empty or too long.
pub fn deactivate_user<C>(
    ctx: &TenantContext,
    user: &mut User,
    cmd: DeactivateUserCommand,
    clock: &C,
) -> Result<UserDeactivated>
where
    C: Clock + ?Sized,
{
    let DeactivateUserCommand {
        tenant,
        user_id,
        reason,
        new_status,
    } = cmd;
    debug_assert_eq!(tenant.school_id, user.school_id);
    let _ = ctx;
    validate_reason(&reason)?;
    let now = clock.now();
    user.status = new_status;
    user.active_status = ActiveStatus::Retired;
    user.updated_at = now;
    user.updated_by = ctx.actor_id;
    user.version = user.version.next();
    let event_id = educore_core::ids::EventId::from_uuid(uuid::Uuid::now_v7());
    user.last_event_id = Some(event_id);
    let event = UserDeactivated::new(
        user_id,
        user.school_id,
        reason,
        new_status,
        event_id,
        ctx.correlation_id,
        now,
    );
    Ok(event)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    dead_code
)]
mod tests {
    use super::*;
    use crate::commands::UniquenessChecker;
    use crate::value_objects::{EmailAddress, HashedPassword};
    use educore_core::clock::{DeterministicIdGen, SystemIdGen, TestClock};
    use educore_core::ids::{SchoolId, UserId};
    use educore_core::tenant::UserType;
    use educore_core::value_objects::Version;
    use educore_events::domain_event::DomainEvent;
    use std::sync::Mutex;

    /// In-memory uniqueness state for the tests.
    #[derive(Default)]
    struct InMemoryUniqueness {
        codes: Mutex<Vec<String>>,
        domains: Mutex<Vec<String>>,
        emails: Mutex<Vec<(SchoolId, String)>>,
        usernames: Mutex<Vec<(SchoolId, String)>>,
    }

    impl InMemoryUniqueness {
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

    fn ctx_for(school: SchoolId) -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SuperAdmin,
        )
    }

    #[test]
    fn create_school_emits_event() {
        let g = DeterministicIdGen::starting_at(0);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::default();
        let school_id = g.next_school_id();
        let ctx_local = ctx_for(school_id);
        let cmd =
            CreateSchoolCommand::new(ctx_local, school_id, "Ada".to_owned(), "ADA".to_owned());
        let (school, event) = create_school(cmd, &clock, &g, &u).unwrap();
        assert_eq!(school.id, school_id);
        assert_eq!(event.school_id(), school_id);
        assert_eq!(
            <SchoolCreated as DomainEvent>::EVENT_TYPE,
            "platform.school.created"
        );
        u.record_school("ADA", None);
    }

    #[test]
    fn update_school_increments_version() {
        let g = DeterministicIdGen::starting_at(100);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::default();
        let school_id = g.next_school_id();
        let ctx_local = ctx_for(school_id);
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
        update_school(&ctx_local, &mut school, upd, &clock, &u).unwrap();
        assert_eq!(school.version, Version::initial().next());
        let upd2 = UpdateSchoolCommand {
            tenant: ctx_local.clone(),
            school_id,
            name: Some("Ada, Countess of Lovelace".to_owned()),
            domain: None,
            package_id: None,
        };
        update_school(&ctx_local, &mut school, upd2, &clock, &u).unwrap();
        assert_eq!(school.version, Version::initial().next().next());
    }

    #[test]
    fn deactivate_school_retires() {
        let g = DeterministicIdGen::starting_at(200);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::default();
        let school_id = g.next_school_id();
        let ctx_local = ctx_for(school_id);
        let cmd = CreateSchoolCommand::new(
            ctx_local.clone(),
            school_id,
            "Ada".to_owned(),
            "ADA".to_owned(),
        );
        let (mut school, _) = create_school(cmd, &clock, &g, &u).unwrap();
        let d = DeactivateSchoolCommand::new(ctx_local.clone(), school_id, "non-payment");
        let _event = deactivate_school(&ctx_local, &mut school, d, &clock).unwrap();
        assert!(school.active_status.is_retired());
        assert_eq!(
            <SchoolDeactivated as DomainEvent>::EVENT_TYPE,
            "platform.school.deactivated"
        );
    }

    #[test]
    fn register_user_emits_event() {
        let g = DeterministicIdGen::starting_at(300);
        let clock = TestClock::new();
        let u = InMemoryUniqueness::default();
        let school_id = g.next_school_id();
        let ctx_local = ctx_for(school_id);
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
        let (user, event) = register_user(cmd, &clock, &g, &u).unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(event.school_id(), school_id);
        assert_eq!(
            <UserRegistered as DomainEvent>::EVENT_TYPE,
            "platform.user.registered"
        );
        assert_eq!(event.aggregate_id(), user_id.as_uuid());
        let _: UserId = user_id;
    }
}
