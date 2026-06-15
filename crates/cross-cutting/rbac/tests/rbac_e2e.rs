//! # RBAC end-to-end tests
//!
//! Integration tests for the `educore-rbac` crate. These tests
//! exercise the public surface of the crate end-to-end and assert
//! on the spec-locked invariants from `docs/specs/rbac/`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]

use std::collections::BTreeSet;
use std::str::FromStr;

use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_rbac::prelude::*;
use educore_rbac::services::InMemoryCapabilityCheck;
use uuid::Uuid;

/// Builds a `TenantContext` for a teacher in a fresh school. The
/// caller can override `user_type` via [`ctx_with_user_type`].
fn ctx_for(user_type: UserType) -> (TenantContext, SchoolId) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let user = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, user, corr, user_type);
    (ctx, school)
}

#[test]
fn capability_parse_and_display_round_trip() {
    for c in Capability::all() {
        let s = c.to_string();
        let parsed = Capability::from_str(&s).unwrap();
        assert_eq!(parsed, *c, "round-trip failed for {c:?}");
    }
}

#[test]
fn capability_unknown_string_rejected() {
    let result = Capability::from_str("Foo.Bar.Baz");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err.kind(),
        educore_core::error::ErrorKind::Validation
    ));
    // The `from_str_opt` helper is the non-erroring counterpart.
    assert!(Capability::from_str_opt("Foo.Bar.Baz").is_none());
}

#[test]
fn super_admin_role_includes_every_capability() {
    let caps = DefaultRoleCatalog::super_admin();
    for c in Capability::all() {
        assert!(caps.contains(c), "SuperAdmin missing {c:?}");
    }
}

#[test]
fn student_role_excludes_finance_capabilities() {
    let caps = DefaultRoleCatalog::student();
    for c in [
        Capability::FinanceInvoiceCreate,
        Capability::FinanceInvoiceRead,
        Capability::FinanceInvoiceUpdate,
        Capability::FinanceInvoiceDelete,
    ] {
        assert!(!caps.contains(&c), "Student should not hold {c:?}");
    }
    // Sanity: students DO see their own profile.
    assert!(caps.contains(&Capability::PlatformUserRead));
}

#[test]
fn in_memory_capability_check_has_returns_true_for_granted() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, Uuid::now_v7());
    let check = InMemoryCapabilityCheck::new();
    check.grant(school, role, Capability::PlatformUserRead);

    let (mut ctx, _) = ctx_for(UserType::Teacher);
    ctx.school_id = school;
    let r = futures::executor::block_on(check.has(&ctx, Capability::PlatformUserRead)).unwrap();
    assert!(r);
}

#[test]
fn in_memory_capability_check_has_returns_false_for_not_granted() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, Uuid::now_v7());
    let check = InMemoryCapabilityCheck::new();
    // Grant an unrelated capability.
    check.grant(school, role, Capability::PlatformUserRead);

    let (mut ctx, _) = ctx_for(UserType::Teacher);
    ctx.school_id = school;
    let r = futures::executor::block_on(check.has(&ctx, Capability::RbacBootstrap)).unwrap();
    assert!(!r, "non-system teacher must not hold RbacBootstrap");
}

#[test]
fn rbac_bootstrap_is_never_revocable() {
    // An actor that holds RbacRoleManage (or is a system actor)
    // implicitly holds RbacBootstrap. The implementation cannot
    // remove the bootstrap capability from SuperAdmin.
    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, Uuid::now_v7());
    let check = InMemoryCapabilityCheck::new();
    check.grant(school, role, Capability::RbacRoleManage);

    // RbacRoleManage holder:
    let (mut ctx_a, _) = ctx_for(UserType::SchoolAdmin);
    ctx_a.school_id = school;
    let r = futures::executor::block_on(check.has(&ctx_a, Capability::RbacBootstrap)).unwrap();
    assert!(r, "RbacRoleManage holder must hold RbacBootstrap");

    // SuperAdmin user type:
    let (mut ctx_b, _) = ctx_for(UserType::SuperAdmin);
    ctx_b.school_id = school;
    let r = futures::executor::block_on(check.has(&ctx_b, Capability::RbacBootstrap)).unwrap();
    assert!(r, "SuperAdmin must hold RbacBootstrap");

    // System user type:
    let g2 = SystemIdGen;
    let sys_ctx = TenantContext::system(school, g2.next_correlation_id());
    let r = futures::executor::block_on(check.has(&sys_ctx, Capability::RbacBootstrap)).unwrap();
    assert!(r, "system actor must hold RbacBootstrap");

    // A plain teacher operating in a *different* school where
    // no role holds RbacRoleManage must NOT hold RbacBootstrap.
    // (The in-memory check sums all roles in the school, so we
    // must use a fresh school to verify the deny path.)
    let other_school = g.next_school_id();
    let (mut ctx_c, _) = ctx_for(UserType::Teacher);
    ctx_c.school_id = other_school;
    let r = futures::executor::block_on(check.has(&ctx_c, Capability::RbacBootstrap)).unwrap();
    assert!(
        !r,
        "a teacher in a school without RbacRoleManage must not hold RbacBootstrap"
    );
}

#[test]
fn is_replicated_flag_round_trips() {
    // The `is_replicated` flag is the prompt's distinguishing field
    // on the `Role` aggregate. Build a role with the flag set,
    // assert it round-trips, and assert that a role without the
    // flag does not acquire it via clone.
    use educore_core::value_objects::{ActiveStatus, Etag, Version};
    use educore_rbac::aggregate::Role;
    use educore_rbac::value_objects::RoleType;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = RoleId::new(school, g.next_uuid());
    let role = Role {
        id,
        school_id: school,
        name: "Replicated Teacher".to_owned(),
        role_type: RoleType::Custom,
        is_system: false,
        is_replicated: true,
        capabilities: BTreeSet::new(),
        version: Version::initial(),
        etag: Etag::new("00000000000000000000000000000001").unwrap(),
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        created_by: g.next_user_id(),
        updated_by: g.next_user_id(),
        active_status: ActiveStatus::Active,
        last_event_id: None,
        correlation_id: g.next_correlation_id(),
    };
    assert!(role.is_replicated);
    let cloned = role.clone();
    assert!(
        cloned.is_replicated,
        "is_replicated must round-trip through Clone"
    );
    // Sanity: a different role created without the flag stays false.
    let mut non_replicated = role.clone();
    non_replicated.id = RoleId::new(school, g.next_uuid());
    non_replicated.is_replicated = false;
    assert!(!non_replicated.is_replicated);
}

#[test]
fn capability_assigned_event_emits_with_correct_event_type() {
    use educore_rbac::events::CapabilityAssigned;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, g.next_uuid());
    let (ctx, _) = ctx_for(UserType::SchoolAdmin);
    let event = CapabilityAssigned::new(
        school,
        role,
        Capability::RbacRoleCreate,
        None,
        Timestamp::now(),
        ctx.correlation_id,
        ctx.actor_id,
    );
    let envelope = event.into_envelope(&ctx);
    assert_eq!(envelope.event_type, "rbac.capability.assigned");
    assert_eq!(envelope.aggregate_type, "rbac_role");
    assert_eq!(envelope.school_id, school);
    assert_eq!(envelope.aggregate_id, role.as_uuid());
    assert_eq!(envelope.actor_id, ctx.actor_id);
}

#[test]
fn capability_revoked_event_has_as_denial_flag() {
    use educore_rbac::events::CapabilityRevoked;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, g.next_uuid());
    let (ctx, _) = ctx_for(UserType::SchoolAdmin);

    // Hard-deletion: the row is removed.
    let hard_delete = CapabilityRevoked::new(
        school,
        role,
        Capability::RbacRoleCreate,
        false,
        Timestamp::now(),
        ctx.correlation_id,
        ctx.actor_id,
    );
    assert!(!hard_delete.as_denial);

    // Explicit-denial: the row is preserved as a denial.
    let mut ctx2 = ctx.clone();
    ctx2.actor_id = g.next_user_id();
    let explicit_deny = CapabilityRevoked::new(
        school,
        role,
        Capability::RbacRoleCreate,
        true,
        Timestamp::now(),
        ctx2.correlation_id,
        ctx2.actor_id,
    );
    assert!(explicit_deny.as_denial);

    let env = explicit_deny.into_envelope(&ctx2);
    assert_eq!(env.event_type, "rbac.capability.revoked");
    assert_eq!(env.payload["as_denial"], serde_json::json!(true));
}

#[test]
fn default_role_catalogs_cover_distinct_responsibility_zones() {
    // Sanity check the cross-role separation: each default role
    // should hold the platform user-read capability (every user
    // can see themselves) but should not hold capabilities
    // outside its scope.
    let teacher = DefaultRoleCatalog::teacher();
    let student = DefaultRoleCatalog::student();
    let accountant = DefaultRoleCatalog::accountant();
    let librarian = DefaultRoleCatalog::librarian();
    let driver = DefaultRoleCatalog::driver();

    // Every role sees its own profile.
    for (name, caps) in [
        ("teacher", &teacher),
        ("student", &student),
        ("accountant", &accountant),
        ("librarian", &librarian),
        ("driver", &driver),
    ] {
        assert!(
            caps.contains(&Capability::PlatformUserRead),
            "{name} should be able to read its own profile"
        );
    }

    // Cross-domain separation: only accountant touches finance,
    // only librarian touches library writes.
    assert!(!teacher.contains(&Capability::FinanceInvoiceCreate));
    assert!(!student.contains(&Capability::BookAdd));
    assert!(!accountant.contains(&Capability::BookAdd));
    assert!(!librarian.contains(&Capability::FinanceInvoiceCreate));

    // The driver has no create/update on the platform user.
    assert!(!driver.contains(&Capability::PlatformUserCreate));
}

#[test]
fn role_created_event_envelope_round_trips() {
    use educore_rbac::events::RoleCreated;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, g.next_uuid());
    let (ctx, _) = ctx_for(UserType::SchoolAdmin);
    let event = RoleCreated::new(
        school,
        role,
        "Teacher".to_owned(),
        RoleType::Custom,
        true,
        Timestamp::now(),
        ctx.correlation_id,
        ctx.actor_id,
    );
    let env = event.into_envelope(&ctx);
    assert_eq!(env.event_type, "rbac.role.created");
    assert_eq!(env.aggregate_type, "rbac_role");
    assert_eq!(env.school_id, school);
    assert_eq!(env.payload["name"], "Teacher");
    assert_eq!(env.payload["is_replicated"], true);
}

#[test]
fn assign_permission_saas_scope_filters_schools() {
    use educore_core::value_objects::{ActiveStatus, Etag, Version};
    use educore_rbac::entities::AssignPermission;
    use educore_rbac::value_objects::{AssignmentStatus, MenuStatus};

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, g.next_uuid());
    let perm = PermissionId::new(school, g.next_uuid());
    let other = SchoolId::from_uuid(Uuid::now_v7());
    let third = SchoolId::from_uuid(Uuid::now_v7());

    let mut saas: BTreeSet<SchoolId> = BTreeSet::new();
    saas.insert(school);
    saas.insert(other);

    let row = AssignPermission {
        id: AssignPermissionId::new(school, g.next_uuid()),
        school_id: school,
        role_id: role,
        permission_id: perm,
        capability: Capability::PlatformUserRead,
        status: AssignmentStatus::Granted,
        menu_status: MenuStatus::Visible,
        saas_schools: Some(saas),
        version: Version::initial(),
        etag: Etag::new("00000000000000000000000000000001").unwrap(),
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        created_by: g.next_user_id(),
        updated_by: g.next_user_id(),
        active_status: ActiveStatus::Active,
        last_event_id: None,
        correlation_id: g.next_correlation_id(),
    };

    assert!(row.applies_to(school));
    assert!(row.applies_to(other));
    assert!(!row.applies_to(third));
    assert!(row.is_granted());
    assert!(!row.is_explicit_denial());
}

#[test]
fn role_service_can_delete_rejects_system_role() {
    use educore_core::value_objects::{ActiveStatus, Etag, Version};
    use educore_rbac::aggregate::Role;
    use educore_rbac::services::RoleService;
    use educore_rbac::value_objects::RoleType;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = Role {
        id: RoleId::new(school, g.next_uuid()),
        school_id: school,
        name: "SuperAdmin".to_owned(),
        role_type: RoleType::System,
        is_system: true,
        is_replicated: true,
        capabilities: DefaultRoleCatalog::super_admin(),
        version: Version::initial(),
        etag: Etag::new("00000000000000000000000000000001").unwrap(),
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        created_by: g.next_user_id(),
        updated_by: g.next_user_id(),
        active_status: ActiveStatus::Active,
        last_event_id: None,
        correlation_id: g.next_correlation_id(),
    };

    let result = RoleService::can_delete(&role, 0);
    assert!(result.is_err(), "system roles must not be deletable");
}

#[test]
fn role_service_can_delete_rejects_role_with_bindings() {
    use educore_core::value_objects::{ActiveStatus, Etag, Version};
    use educore_rbac::aggregate::Role;
    use educore_rbac::services::RoleService;
    use educore_rbac::value_objects::RoleType;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = Role {
        id: RoleId::new(school, g.next_uuid()),
        school_id: school,
        name: "Teacher".to_owned(),
        role_type: RoleType::Custom,
        is_system: false,
        is_replicated: false,
        capabilities: BTreeSet::new(),
        version: Version::initial(),
        etag: Etag::new("00000000000000000000000000000001").unwrap(),
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        created_by: g.next_user_id(),
        updated_by: g.next_user_id(),
        active_status: ActiveStatus::Active,
        last_event_id: None,
        correlation_id: g.next_correlation_id(),
    };

    assert!(RoleService::can_delete(&role, 0).is_ok());
    let err = RoleService::can_delete(&role, 1).unwrap_err();
    assert!(matches!(
        err.kind(),
        educore_core::error::ErrorKind::Conflict
    ));
}

#[test]
fn capability_check_has_any_and_has_all() {
    use std::collections::BTreeSet;

    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, Uuid::now_v7());
    let check = InMemoryCapabilityCheck::new();
    // Grant a couple of capabilities.
    let mut granted: BTreeSet<Capability> = BTreeSet::new();
    granted.insert(Capability::PlatformUserRead);
    granted.insert(Capability::RbacCapabilityRead);
    for c in &granted {
        check.grant(school, role, *c);
    }

    let (mut ctx, _) = ctx_for(UserType::Teacher);
    ctx.school_id = school;

    // has_any returns true if at least one matches.
    let r = futures::executor::block_on(check.has_any(
        &ctx,
        &[Capability::RbacRoleCreate, Capability::PlatformUserRead],
    ))
    .unwrap();
    assert!(r);

    // has_all returns false if any is missing.
    let r = futures::executor::block_on(check.has_all(
        &ctx,
        &[Capability::PlatformUserRead, Capability::RbacRoleCreate],
    ))
    .unwrap();
    assert!(!r);

    // has_all returns true if all match.
    let r = futures::executor::block_on(check.has_all(
        &ctx,
        &[Capability::PlatformUserRead, Capability::RbacCapabilityRead],
    ))
    .unwrap();
    assert!(r);
}

#[test]
fn capability_explain_includes_role_grants() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let role = RoleId::new(school, Uuid::now_v7());
    let check = InMemoryCapabilityCheck::new();
    check.grant(school, role, Capability::PlatformUserRead);

    let (mut ctx, _) = ctx_for(UserType::Teacher);
    ctx.school_id = school;
    let exp =
        futures::executor::block_on(check.explain(&ctx, Capability::PlatformUserRead)).unwrap();
    assert!(exp.decision);
    assert!(exp.role_grants.contains(&role));
    assert!(!exp.system_fallback);
}

#[test]
fn command_types_carry_tenant_context_and_target_role() {
    // The command shapes carry the `TenantContext` (school + actor +
    // correlation) and the target `RoleId`. This is the engine's
    // typed boundary — no string fields.
    let g = SystemIdGen;
    let ctx = {
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        TenantContext::for_user(school, user, corr, UserType::SchoolAdmin)
    };
    let school = ctx.school_id;
    let role = RoleId::new(school, g.next_uuid());
    let user: UserId = g.next_user_id();

    let create = CreateRoleCommand {
        tenant: ctx.clone(),
        name: RoleName::new("Auditor").unwrap(),
        role_type: RoleType::Custom,
        is_replicated: false,
    };
    assert_eq!(create.school_id(), school);

    let update = UpdateRoleCommand {
        tenant: ctx.clone(),
        role_id: role,
        name: Some(RoleName::new("Senior Auditor").unwrap()),
        is_replicated: Some(true),
    };
    assert_eq!(update.school_id(), school);
    assert_eq!(update.role_id, role);

    let delete = DeleteRoleCommand {
        tenant: ctx.clone(),
        role_id: role,
    };
    assert_eq!(delete.role_id, role);

    let assign = AssignCapabilityCommand {
        tenant: ctx.clone(),
        role_id: role,
        capability: Capability::RbacCapabilityRead,
        saas_schools: None,
    };
    assert_eq!(assign.role_id, role);
    assert_eq!(assign.capability, Capability::RbacCapabilityRead);

    let revoke = RevokeCapabilityCommand {
        tenant: ctx.clone(),
        role_id: role,
        capability: Capability::RbacCapabilityRead,
        as_denial: true,
    };
    assert!(revoke.as_denial);
    assert_eq!(revoke.role_id, role);
    let _ = user;
}

#[test]
fn repository_port_trait_is_object_safe() {
    // Object-safety compile test: verify that the trait can be used
    // as a `dyn Trait`. This catches accidental `Self` bounds or
    // generic methods that would break the trait's vtable.
    fn _is_object_safe(_: &dyn RoleRepository) {}
    fn _is_object_safe_assign(_: &dyn AssignPermissionRepository) {}
    fn _is_object_safe_perm(_: &dyn PermissionRepository) {}
    fn _is_object_safe_section(_: &dyn PermissionSectionRepository) {}
}
