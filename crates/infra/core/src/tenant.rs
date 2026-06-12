//! The `TenantContext` — the active tenant for a single command or
//! query.
//!
//! Per `docs/schemas/tenancy-schema.md` § 3:
//!
//! - The `TenantContext` is **immutable** for the lifetime of a single
//!   command.
//! - It is constructed at the engine boundary (the consumer's
//!   authentication layer).
//! - It is **never** constructed by domain code. Domain code receives
//!   it as input and reads `school_id` from it.
//! - For background jobs, `actor_id` is the job's service user; the
//!   `correlation_id` is the job's run id.
//!
//! The cross-cutting presentation fields (`locale`, `timezone`) are
//! string newtypes over standard IANA / BCP 47 codes; the engine
//! does not parse or normalize them — that is the consumer's job at
//! the engine boundary.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ids::{CorrelationId, EventId, SchoolId, SessionId, UserId, SYSTEM_USER_ID};

/// The active tenant for a single command or query.
///
/// Construct only at the engine boundary. Domain code receives a
/// `&TenantContext` and reads [`TenantContext::school_id`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantContext {
    /// The active school. Mandatory.
    pub school_id: SchoolId,
    /// The active user. Use [`TenantContext::system`] for
    /// system-issued commands (jobs, migrations).
    pub actor_id: UserId,
    /// Optional session boundary.
    pub session_id: Option<SessionId>,
    /// Propagated to every event emitted by the command.
    pub correlation_id: CorrelationId,
    /// For chained commands, the id of the event that caused this
    /// command. `None` for top-level commands.
    pub causation_id: Option<EventId>,
    /// The actor's role. Drives default RBAC bindings and per-domain
    /// capability checks.
    pub user_type: UserType,
    /// Presentation locale (BCP 47, e.g. `en-US`). Storage layer
    /// never reads this; the engine uses it only for rendering
    /// default values.
    pub locale: Locale,
    /// Presentation timezone (IANA, e.g. `America/Los_Angeles`).
    /// Storage layer never reads this.
    pub timezone: TimeZone,
}

impl TenantContext {
    /// Constructs a `TenantContext` for a real authenticated user.
    #[must_use]
    pub fn for_user(
        school_id: SchoolId,
        actor_id: UserId,
        correlation_id: CorrelationId,
        user_type: UserType,
    ) -> Self {
        Self {
            school_id,
            actor_id,
            session_id: None,
            correlation_id,
            causation_id: None,
            user_type,
            locale: Locale::default(),
            timezone: TimeZone::default(),
        }
    }

    /// Constructs a `TenantContext` for a system-issued command
    /// (background job, migration, scheduled task). The actor is
    /// the engine's [`SYSTEM_USER_ID`] and the user type is
    /// [`UserType::System`].
    #[must_use]
    pub fn system(school_id: SchoolId, correlation_id: CorrelationId) -> Self {
        Self {
            school_id,
            actor_id: SYSTEM_USER_ID,
            session_id: None,
            correlation_id,
            causation_id: None,
            user_type: UserType::System,
            locale: Locale::default(),
            timezone: TimeZone::default(),
        }
    }

    /// Returns a builder for the remaining optional fields.
    #[must_use]
    pub fn builder(self) -> TenantContextBuilder {
        TenantContextBuilder { inner: self }
    }
}

/// Builder for the optional fields of a `TenantContext`. Use
/// [`TenantContext::for_user`] or [`TenantContext::system`] to
/// construct the required fields, then `.builder()` to set the
/// optional ones.
#[derive(Debug, Clone)]
pub struct TenantContextBuilder {
    inner: TenantContext,
}

impl TenantContextBuilder {
    /// Sets the session id.
    #[must_use]
    pub fn session_id(mut self, id: SessionId) -> Self {
        self.inner.session_id = Some(id);
        self
    }

    /// Sets the causation id.
    #[must_use]
    pub fn causation_id(mut self, id: EventId) -> Self {
        self.inner.causation_id = Some(id);
        self
    }

    /// Sets the presentation locale.
    #[must_use]
    pub fn locale(mut self, locale: Locale) -> Self {
        self.inner.locale = locale;
        self
    }

    /// Sets the presentation timezone.
    #[must_use]
    pub fn timezone(mut self, tz: TimeZone) -> Self {
        self.inner.timezone = tz;
        self
    }

    /// Returns the built context.
    #[must_use]
    pub fn build(self) -> TenantContext {
        self.inner
    }
}

/// The role of the actor in the active school. Per
/// `docs/specs/platform/value-objects.md` (UserType) and
/// `docs/schemas/tenancy-schema.md` § 3.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserType {
    /// A platform operator with cross-school authority.
    SuperAdmin,
    /// The school's administrator.
    SchoolAdmin,
    /// A teaching staff member.
    Teacher,
    /// An enrolled student.
    Student,
    /// A parent or guardian of a student.
    Parent,
    /// A finance / accounting staff member.
    Accountant,
    /// A library staff member.
    Librarian,
    /// A front-desk / reception staff member.
    Receptionist,
    /// Generic non-teaching staff.
    #[default]
    Staff,
    /// A transport driver.
    Driver,
    /// A parent who is also a fee-paying customer of the school.
    Customer,
    /// A system-issued actor (job, migration, scheduled task).
    System,
}

impl UserType {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SuperAdmin => "super_admin",
            Self::SchoolAdmin => "school_admin",
            Self::Teacher => "teacher",
            Self::Student => "student",
            Self::Parent => "parent",
            Self::Accountant => "accountant",
            Self::Librarian => "librarian",
            Self::Receptionist => "receptionist",
            Self::Staff => "staff",
            Self::Driver => "driver",
            Self::Customer => "customer",
            Self::System => "system",
        }
    }

    /// Returns `true` if the user type is a school-scoped actor
    /// (i.e. lives inside a single school, not a platform operator).
    #[must_use]
    pub const fn is_school_scoped(self) -> bool {
        !matches!(self, Self::SuperAdmin)
    }
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A presentation locale (BCP 47, e.g. `en-US`). The engine does
/// not validate the tag — consumers normalize at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Locale(String);

impl Locale {
    /// Constructs a `Locale` from a raw string. The string is not
    /// validated against the IANA / BCP 47 registry; the engine
    /// treats the value as opaque.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the locale as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self("en".to_owned())
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Locale {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A presentation timezone (IANA, e.g. `America/Los_Angeles`). The
/// engine does not validate the tag — consumers normalize at the
/// boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeZone(String);

impl TimeZone {
    /// Constructs a `TimeZone` from a raw string. The string is not
    /// validated against the IANA tz database; the engine treats
    /// the value as opaque.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the timezone as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the canonical "UTC" timezone constant.
    #[must_use]
    pub fn utc() -> Self {
        Self("UTC".to_owned())
    }
}

impl Default for TimeZone {
    fn default() -> Self {
        Self::utc()
    }
}

impl fmt::Display for TimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for TimeZone {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
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
    use crate::clock::IdGenerator;
    use crate::clock::SystemIdGen;
    use crate::ids::Identifier;

    #[test]
    fn for_user_constructs_minimal_context() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let ctx = TenantContext::for_user(school, user, corr, UserType::Teacher);
        assert_eq!(ctx.school_id, school);
        assert_eq!(ctx.actor_id, user);
        assert_eq!(ctx.correlation_id, corr);
        assert_eq!(ctx.user_type, UserType::Teacher);
        assert_eq!(ctx.session_id, None);
        assert_eq!(ctx.causation_id, None);
    }

    #[test]
    fn system_context_uses_system_user() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let corr = g.next_correlation_id();
        let ctx = TenantContext::system(school, corr);
        assert_eq!(ctx.actor_id, SYSTEM_USER_ID);
        assert_eq!(ctx.user_type, UserType::System);
    }

    #[test]
    fn builder_sets_optional_fields() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let session = g.next_session_id();
        let cause = g.next_event_id();
        let ctx = TenantContext::for_user(school, user, corr, UserType::Parent)
            .builder()
            .session_id(session)
            .causation_id(cause)
            .locale(Locale::new("fr-FR"))
            .timezone(TimeZone::new("Europe/Paris"))
            .build();
        assert_eq!(ctx.session_id, Some(session));
        assert_eq!(ctx.causation_id, Some(cause));
        assert_eq!(ctx.locale.as_str(), "fr-FR");
        assert_eq!(ctx.timezone.as_str(), "Europe/Paris");
    }

    #[test]
    fn user_type_round_trip() {
        for ut in [
            UserType::SuperAdmin,
            UserType::SchoolAdmin,
            UserType::Teacher,
            UserType::Student,
            UserType::Parent,
            UserType::Accountant,
            UserType::Librarian,
            UserType::Receptionist,
            UserType::Staff,
            UserType::Driver,
            UserType::Customer,
            UserType::System,
        ] {
            assert!(!ut.as_str().is_empty());
        }
        assert!(!UserType::SuperAdmin.is_school_scoped());
        assert!(UserType::Teacher.is_school_scoped());
    }

    #[test]
    fn locale_and_timezone_defaults() {
        assert_eq!(Locale::default().as_str(), "en");
        assert_eq!(TimeZone::default().as_str(), "UTC");
    }

    #[test]
    fn system_user_id_is_v7_marker() {
        assert_eq!(SYSTEM_USER_ID.as_uuid().get_version_num(), 7);
    }
}
