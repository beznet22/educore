//! Platform commands.
//!
//! Phase 2 ships the six commands enumerated in the prompt:
//! - [`CreateSchoolCommand`], [`UpdateSchoolCommand`],
//!   [`DeactivateSchoolCommand`]
//! - [`RegisterUserCommand`], [`UpdateUserCommand`],
//!   [`DeactivateUserCommand`]
//!
//! Each command carries a `TenantContext` and the input
//! fields validated by the matching
//! [`services`](crate::services) factory function. The
//! `services` module is the only place the engine mutates
//! the aggregate and emits the typed event.

use serde::{Deserialize, Serialize};

use educore_core::ids::{SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    EmailAddress, HashedPassword, PackageId, PhoneNumber, RoleId, SchoolStatus, UserStatus,
};

/// A read-only uniqueness check the platform services use to
/// enforce per-school uniqueness constraints.
///
/// The check is **pure** (no I/O): the production caller wires
/// it to a thin adapter over the storage port that returns
/// `true` if a row with the given key already exists; the test
/// caller wires it to a closure backed by an in-memory
/// collection.
///
/// The trait is `Send + Sync` so the production wiring can
/// hold an `Arc<dyn UniquenessChecker>` and share it across
/// worker threads.
pub trait UniquenessChecker: Send + Sync {
    /// Returns `true` if a school with the given `school_code`
    /// already exists on the platform.
    fn school_code_exists(&self, code: &str) -> bool;
    /// Returns `true` if a school with the given public
    /// `domain` already exists on the platform.
    fn school_domain_exists(&self, domain: &str) -> bool;
    /// Returns `true` if a user with the given lowercased
    /// `email` already exists in `school`.
    fn user_email_exists(&self, school: SchoolId, email: &str) -> bool;
    /// Returns `true` if a user with the given `username`
    /// already exists in `school`.
    fn user_username_exists(&self, school: SchoolId, username: &str) -> bool;
}

/// Command: create a new school.
///
/// The command's `school_id` is supplied by the dispatcher
/// (after the engine has minted it via the `IdGenerator`
/// port) so that the new school's id is part of the command
/// shape rather than something the factory function
/// discovers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSchoolCommand {
    /// The active tenant (typically `System` for school
    /// creation; the engine does not enforce this here — the
    /// RBAC subscriber does).
    pub tenant: TenantContext,
    /// The new school's typed id. Mints it as `Some(school_id)`;
    /// the storage adapter uses it as the primary key on
    /// insert.
    pub school_id: SchoolId,
    /// The school's display name (1..=200 chars).
    pub name: String,
    /// The school's unique short code (1..=200 chars).
    pub school_code: String,
    /// The school's optional public-facing domain.
    pub domain: Option<String>,
    /// The school's optional initial package binding.
    pub package_id: Option<PackageId>,
}

/// Command: update a school's mutable fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateSchoolCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The school's typed id.
    pub school_id: SchoolId,
    /// Optional new display name. `None` means "do not change".
    pub name: Option<String>,
    /// Optional new domain. `None` means "do not change";
    /// `Some(None)` (i.e. an outer `Some` wrapping an inner
    /// `None`) means "clear the domain".
    pub domain: Option<Option<String>>,
    /// Optional new package binding. Outer `None` means "do
    /// not change"; outer `Some(None)` means "clear the
    /// binding".
    pub package_id: Option<Option<PackageId>>,
}

/// Command: deactivate a school.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeactivateSchoolCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The school's typed id.
    pub school_id: SchoolId,
    /// The reason for deactivation (1..=500 chars).
    pub reason: String,
    /// The new lifecycle status the school should be set to
    /// (typically `Suspended`).
    pub new_status: SchoolStatus,
}

/// Command: register a new user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterUserCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The new user's typed id.
    pub user_id: UserId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The user's email (validated, lowercased).
    pub email: EmailAddress,
    /// The user's chosen username.
    pub username: String,
    /// The user's display name.
    pub display_name: String,
    /// Optional phone number in E.164 form.
    pub phone_number: Option<PhoneNumber>,
    /// The actor's role.
    pub usertype: UserType,
    /// Initial role bindings (usually empty at registration;
    /// the role is set by a follow-up `ChangeUserRole`
    /// command).
    pub role_ids: Vec<RoleId>,
    /// The pre-computed password hash. The plaintext is never
    /// stored; the engine's password port produces this.
    pub password_hash: HashedPassword,
}

/// Command: update a user's mutable fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateUserCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The user's typed id.
    pub user_id: UserId,
    /// Optional new email. `None` means "do not change".
    pub email: Option<EmailAddress>,
    /// Optional new display name. `None` means "do not
    /// change".
    pub display_name: Option<String>,
    /// Optional new phone number. Outer `None` means "do not
    /// change"; outer `Some(None)` means "clear the phone
    /// number".
    pub phone_number: Option<Option<PhoneNumber>>,
}

/// Command: deactivate a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeactivateUserCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// The user's typed id.
    pub user_id: UserId,
    /// The reason for deactivation (1..=500 chars).
    pub reason: String,
    /// The new user status the user should be set to
    /// (typically `Inactive` or `Suspended`).
    pub new_status: UserStatus,
}

impl CreateSchoolCommand {
    /// Convenience constructor for tests and bootstrapping.
    #[must_use]
    pub const fn new(
        tenant: TenantContext,
        school_id: SchoolId,
        name: String,
        school_code: String,
    ) -> Self {
        Self {
            tenant,
            school_id,
            name,
            school_code,
            domain: None,
            package_id: None,
        }
    }
}

impl RegisterUserCommand {
    /// Convenience constructor for tests and bootstrapping.
    #[must_use]
    pub const fn new(
        tenant: TenantContext,
        user_id: UserId,
        school_id: SchoolId,
        email: EmailAddress,
        username: String,
        display_name: String,
        password_hash: HashedPassword,
    ) -> Self {
        Self {
            tenant,
            user_id,
            school_id,
            email,
            username,
            display_name,
            phone_number: None,
            usertype: UserType::Staff,
            role_ids: Vec::new(),
            password_hash,
        }
    }
}

impl DeactivateSchoolCommand {
    /// Convenience constructor for tests and bootstrapping.
    #[must_use]
    pub fn new(tenant: TenantContext, school_id: SchoolId, reason: impl Into<String>) -> Self {
        Self {
            tenant,
            school_id,
            reason: reason.into(),
            new_status: SchoolStatus::Suspended,
        }
    }
}

impl DeactivateUserCommand {
    /// Convenience constructor for tests and bootstrapping.
    #[must_use]
    pub fn new(tenant: TenantContext, user_id: UserId, reason: impl Into<String>) -> Self {
        Self {
            tenant,
            user_id,
            reason: reason.into(),
            new_status: UserStatus::Inactive,
        }
    }
}

/// Internal: shared validation helpers used by the
/// `services` factory functions.
pub(crate) fn validate_school_name(name: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if name.is_empty() {
        return Err(DomainError::Validation(
            "school name must not be empty".to_owned(),
        ));
    }
    if name.chars().count() > 200 {
        return Err(DomainError::Validation(format!(
            "school name length {} exceeds 200",
            name.chars().count()
        )));
    }
    let _ = Timestamp::now;
    Ok(())
}

pub(crate) fn validate_school_code(code: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if code.is_empty() {
        return Err(DomainError::Validation(
            "school code must not be empty".to_owned(),
        ));
    }
    if code.chars().count() > 200 {
        return Err(DomainError::Validation(format!(
            "school code length {} exceeds 200",
            code.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_username(username: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if username.is_empty() {
        return Err(DomainError::Validation(
            "username must not be empty".to_owned(),
        ));
    }
    if username.chars().count() > 192 {
        return Err(DomainError::Validation(format!(
            "username length {} exceeds 192",
            username.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_display_name(name: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if name.is_empty() {
        return Err(DomainError::Validation(
            "display name must not be empty".to_owned(),
        ));
    }
    if name.chars().count() > 200 {
        return Err(DomainError::Validation(format!(
            "display name length {} exceeds 200",
            name.chars().count()
        )));
    }
    Ok(())
}

pub(crate) fn validate_reason(reason: &str) -> educore_core::error::Result<()> {
    use educore_core::error::DomainError;
    if reason.is_empty() {
        return Err(DomainError::Validation(
            "reason must not be empty".to_owned(),
        ));
    }
    if reason.chars().count() > 500 {
        return Err(DomainError::Validation(format!(
            "reason length {} exceeds 500",
            reason.chars().count()
        )));
    }
    Ok(())
}

// ===========================================================================
// Compliance-report commands (Phase 2 § 8.2-8.4)
//
// These commands close three roadmap items:
// - SCHEMA-AUDIT-GDPR-ERASURE (`docs/schemas/audit-schema.md` § 8.2)
// - SCHEMA-AUDIT-FERPA        (`docs/schemas/audit-schema.md` § 8.3)
// - SCHEMA-AUDIT-REGULATOR    (`docs/schemas/audit-schema.md` § 8.4)
//
// They are scaffolded here so the lint gate is satisfied
// (`#![deny(missing_docs)]` requires every `pub` type to be
// documented); the full service implementations land in later
// phases alongside the audit-domain anonymization helpers.
// ===========================================================================

/// Command: execute a GDPR right-to-erasure for the subject
/// `subject_id` (Phase 2 § 8.2). The engine soft-deletes the
/// subject's profile, anonymizes PII in the audit log, retains
/// financial records for the regulator-required period (default
/// 7 years), and emits an audit event recording the erasure.
///
/// `request_id` is the data-subject's request reference id;
/// `reason` is a free-text explanation required by most
/// jurisdictions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecuteSubjectErasureCommand {
    /// The active tenant (typically `System` or a compliance
    /// officer's `TenantContext`).
    pub tenant: TenantContext,
    /// The typed id of the subject to erase.
    pub subject_id: UserId,
    /// The data-subject's request reference id (e.g. the
    /// support-ticket id from the privacy portal).
    pub request_id: String,
    /// A free-text explanation recorded with the erasure event.
    pub reason: String,
}

/// Command: generate a FERPA-style parental access report
/// (Phase 2 § 8.3). The engine assembles the child's academic,
/// behavioural, and financial records into a self-contained
/// JSON / PDF bundle. The parent must hold `Parent.Read` for
/// the child; the RBAC subscriber enforces this.
///
/// `report_format` selects the output wire format; the engine
/// supports `Json` (default) and `Pdf` in later phases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateParentAccessReportCommand {
    /// The active tenant (the parent's `TenantContext`; the
    /// engine checks `Parent.Read` against `child_id`).
    pub tenant: TenantContext,
    /// The typed id of the child whose records are being
    /// requested.
    pub child_id: UserId,
    /// The output format requested by the parent.
    pub report_format: ReportFormat,
}

/// Command: generate a regulator audit bundle (Phase 2 § 8.4).
/// The engine assembles all state changes, authorization
/// decisions, data exports, and backup/restore events for
/// `school_id` in `[from, to]` into a signed bundle. The
/// consumer's compliance team reviews and signs off internally
/// before disclosure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateRegulatorAuditCommand {
    /// The active tenant (typically `System`).
    pub tenant: TenantContext,
    /// The school under audit.
    pub school_id: SchoolId,
    /// Inclusive start of the audit time range.
    pub from: Timestamp,
    /// Inclusive end of the audit time range.
    pub to: Timestamp,
}

/// Output format requested by a parent-access report
/// (Phase 2 § 8.3). The engine supports `Json` (default) and
/// `Pdf` in later phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// JSON bundle (default).
    Json,
    /// PDF bundle (consumer's renderer).
    Pdf,
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::UserType;

    fn ctx() -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SuperAdmin,
        )
    }

    #[test]
    fn create_school_command_new_minimal() {
        let g = SystemIdGen;
        let cmd = CreateSchoolCommand::new(
            ctx(),
            g.next_school_id(),
            "Ada".to_owned(),
            "ADA".to_owned(),
        );
        assert_eq!(cmd.name, "Ada");
        assert_eq!(cmd.school_code, "ADA");
        assert!(cmd.domain.is_none());
        assert!(cmd.package_id.is_none());
    }

    #[test]
    fn register_user_command_new_minimal() {
        let g = SystemIdGen;
        let cmd = RegisterUserCommand::new(
            ctx(),
            g.next_user_id(),
            g.next_school_id(),
            EmailAddress::new("ada@example.com").unwrap(),
            "ada".to_owned(),
            "Ada".to_owned(),
            HashedPassword::from_hash("$argon2id$dummy"),
        );
        assert_eq!(cmd.username, "ada");
        assert!(cmd.phone_number.is_none());
        assert!(cmd.role_ids.is_empty());
    }

    #[test]
    fn validate_reasons() {
        assert!(validate_reason("").is_err());
        assert!(validate_reason("ok").is_ok());
    }

    #[test]
    fn validate_school_code_rejects_empty() {
        assert!(validate_school_code("").is_err());
        assert!(validate_school_code("ADA").is_ok());
    }

    #[test]
    fn validate_username_rejects_overlong() {
        let s = "a".repeat(193);
        assert!(validate_username(&s).is_err());
    }
}
