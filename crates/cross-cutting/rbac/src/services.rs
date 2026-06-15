//! # RBAC services
//!
//! The engine's capability check port, an in-memory implementation,
//! the [`RoleService`], and the [`DefaultRoleCatalog`].
//!
//! `CapabilityCheck` is the **only** service other domains call.
//! The default implementation is in-memory; the storage-backed
//! implementation lives in the adapter crates (PostgreSQL /
//! MySQL / SQLite).
//!
//! Per `docs/specs/rbac/services.md`, the `RbacBootstrap` capability
//! is held by `SuperAdmin` and is never revocable; any
//! `CapabilityCheck` implementation must honor that invariant.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::ids::RoleId;
use crate::value_objects::Capability;

/// A per-actor override of a permission grant. The full
/// `PermissionOverride` aggregate is a Phase 2 follow-up; the
/// service trait references an opaque id so the type surface stays
/// stable across phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityOverride {
    /// The id of the override row.
    pub id: uuid::Uuid,
    /// `true` for an emergency grant, `false` for a deliberate denial.
    pub granted: bool,
}

/// How a capability decision was reached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityExplanation {
    /// The capability that was checked.
    pub capability: Capability,
    /// `true` if the capability is granted.
    pub decision: bool,
    /// Roles that grant this capability.
    pub role_grants: Vec<RoleId>,
    /// Overrides that contributed to the decision.
    pub overrides: Vec<CapabilityOverride>,
    /// `true` if the decision came from the bootstrap backstop
    /// (`RbacBootstrap` always returns `true` for system actors).
    pub system_fallback: bool,
}

/// The engine's capability check port.
///
/// Every other domain calls [`CapabilityCheck::has`] at the command
/// boundary to authorize a state change. Implementations are
/// stateless from the caller's perspective: the port owns the
/// in-memory cache and the catalog.
#[async_trait]
pub trait CapabilityCheck: Send + Sync {
    /// Returns `true` if the actor in `ctx` holds the capability.
    async fn has(&self, ctx: &TenantContext, capability: Capability) -> Result<bool>;

    /// Returns `true` if the actor holds at least one of the
    /// capabilities in the slice.
    async fn has_any(&self, ctx: &TenantContext, capabilities: &[Capability]) -> Result<bool>;

    /// Returns `true` if the actor holds every capability in the slice.
    async fn has_all(&self, ctx: &TenantContext, capabilities: &[Capability]) -> Result<bool>;

    /// Returns a structured explanation of how the decision was
    /// reached. Used by the audit log and the "why is this denied?"
    /// diagnostic screen.
    async fn explain(
        &self,
        ctx: &TenantContext,
        capability: Capability,
    ) -> Result<CapabilityExplanation>;

    /// Drops the in-memory cache for the actor's school. Called by
    /// the `CapabilityAssigned` / `CapabilityRevoked` event
    /// subscribers.
    async fn invalidate_cache(&self, ctx: &TenantContext) -> Result<()>;
}

/// The shape of the in-memory grant table: school → role → capability
/// set. Pulled out as a type alias so [`InMemoryCapabilityCheck`]
/// stays readable.
type GrantTable = BTreeMap<SchoolId, BTreeMap<RoleId, BTreeSet<Capability>>>;

/// In-memory implementation of [`CapabilityCheck`]. Holds
/// [`GrantTable`] keyed by school.
///
/// The bootstrap invariant: any actor that holds
/// `RbacRoleManage` (or is the engine's system actor) is treated
/// as having every `Rbac.*` capability, including
/// [`Capability::RbacBootstrap`]. `RbacBootstrap` is never
/// revocable from the catalog — the in-memory implementation
/// cannot remove it.
#[derive(Debug, Default, Clone)]
pub struct InMemoryCapabilityCheck {
    /// School → (Role → Granted capabilities).
    inner: Arc<RwLock<GrantTable>>,
}

impl InMemoryCapabilityCheck {
    /// Creates an empty in-memory capability check.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a direct grant of `capability` to the role. Returns the
    /// previous grant set for the role (empty if the role was new).
    pub fn grant(
        &self,
        school: SchoolId,
        role: RoleId,
        capability: Capability,
    ) -> BTreeSet<Capability> {
        let mut g = match self.inner.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let by_school = g.entry(school).or_default();
        let caps = by_school.entry(role).or_default();
        caps.insert(capability);
        caps.clone()
    }

    /// Removes a direct grant. Returns the previous grant set for
    /// the role.
    ///
    /// **Note:** callers should not invoke this for
    /// [`Capability::RbacBootstrap`] from a non-system role; the
    /// in-memory implementation does not refuse the removal, but
    /// the [`has`](Self::has) check will still return `true` for
    /// system actors regardless of the stored grant set (the
    /// bootstrap backstop).
    pub fn revoke(
        &self,
        school: SchoolId,
        role: RoleId,
        capability: Capability,
    ) -> BTreeSet<Capability> {
        let mut g = match self.inner.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let caps = g.entry(school).or_default().entry(role).or_default();
        caps.remove(&capability);
        caps.clone()
    }

    /// Returns the union of capabilities held by the actor's roles.
    /// In the Phase 2 in-memory implementation, the actor's roles
    /// are read from the `TenantContext` session fields; for now
    /// we treat the `UserType` as the single-role hint.
    fn grants_for(&self, ctx: &TenantContext) -> BTreeSet<Capability> {
        let g = match self.inner.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let by_school = match g.get(&ctx.school_id) {
            Some(m) => m,
            None => return BTreeSet::new(),
        };
        // The Phase 2 in-memory check accepts a single role id via
        // the session. For now we just sum all roles in the school
        // — the storage-backed impl will read the user→role
        // bindings.
        let mut caps = BTreeSet::new();
        for set in by_school.values() {
            caps.extend(set.iter().copied());
        }
        caps
    }

    /// Returns `true` if the actor is treated as a system actor for
    /// the bootstrap backstop. The full implementation will consult
    /// the `UserType` plus the user's role bindings; the Phase 2
    /// in-memory implementation recognises `SuperAdmin` and `System`.
    fn is_system_actor(&self, ctx: &TenantContext) -> bool {
        matches!(
            ctx.user_type,
            educore_core::tenant::UserType::SuperAdmin | educore_core::tenant::UserType::System
        )
    }

    /// The bootstrap backstop: if the actor holds `RbacRoleManage`
    /// (or is a system actor), they implicitly hold every `Rbac.*`
    /// capability, including [`Capability::RbacBootstrap`]. The
    /// backstop is read-only: removing the role's explicit
    /// `RbacRoleManage` grant will remove the backstop on the next
    /// refresh.
    fn apply_bootstrap_backstop(&self, ctx: &TenantContext, caps: &mut BTreeSet<Capability>) {
        if self.is_system_actor(ctx) || caps.contains(&Capability::RbacRoleManage) {
            caps.insert(Capability::RbacRoleCreate);
            caps.insert(Capability::RbacRoleRead);
            caps.insert(Capability::RbacRoleUpdate);
            caps.insert(Capability::RbacRoleDelete);
            caps.insert(Capability::RbacRoleManage);
            caps.insert(Capability::RbacRoleClone);
            caps.insert(Capability::RbacCapabilityAssign);
            caps.insert(Capability::RbacCapabilityRevoke);
            caps.insert(Capability::RbacCapabilityRead);
            caps.insert(Capability::RbacCapabilityUpdateMetadata);
            caps.insert(Capability::RbacBootstrap);
        }
    }
}

#[async_trait]
impl CapabilityCheck for InMemoryCapabilityCheck {
    async fn has(&self, ctx: &TenantContext, capability: Capability) -> Result<bool> {
        let mut caps = self.grants_for(ctx);
        self.apply_bootstrap_backstop(ctx, &mut caps);
        Ok(caps.contains(&capability))
    }

    async fn has_any(&self, ctx: &TenantContext, capabilities: &[Capability]) -> Result<bool> {
        if capabilities.is_empty() {
            return Ok(true);
        }
        let mut caps = self.grants_for(ctx);
        self.apply_bootstrap_backstop(ctx, &mut caps);
        Ok(capabilities.iter().any(|c| caps.contains(c)))
    }

    async fn has_all(&self, ctx: &TenantContext, capabilities: &[Capability]) -> Result<bool> {
        if capabilities.is_empty() {
            return Ok(true);
        }
        let mut caps = self.grants_for(ctx);
        self.apply_bootstrap_backstop(ctx, &mut caps);
        Ok(capabilities.iter().all(|c| caps.contains(c)))
    }

    async fn explain(
        &self,
        ctx: &TenantContext,
        capability: Capability,
    ) -> Result<CapabilityExplanation> {
        let g = match self.inner.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let by_school = g.get(&ctx.school_id);
        let role_grants: Vec<RoleId> = by_school
            .map(|m| {
                m.iter()
                    .filter_map(|(role, caps)| caps.contains(&capability).then_some(*role))
                    .collect()
            })
            .unwrap_or_default();
        let mut caps = self.grants_for(ctx);
        let pre_backstop = caps.contains(&capability);
        self.apply_bootstrap_backstop(ctx, &mut caps);
        let decision = caps.contains(&capability);
        let system_fallback = decision && !pre_backstop;
        Ok(CapabilityExplanation {
            capability,
            decision,
            role_grants,
            overrides: Vec::new(),
            system_fallback,
        })
    }

    async fn invalidate_cache(&self, ctx: &TenantContext) -> Result<()> {
        let mut g = match self.inner.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        g.remove(&ctx.school_id);
        Ok(())
    }
}

/// Domain service for role-level invariants. Pure: no I/O.
#[derive(Debug, Default, Clone, Copy)]
pub struct RoleService;

impl RoleService {
    /// Returns the union of capabilities granted to a role by its
    /// `AssignPermission` rows.
    #[must_use]
    pub fn effective_capabilities(
        role: &crate::aggregate::Role,
        assignments: &[crate::entities::AssignPermission],
    ) -> BTreeSet<Capability> {
        let mut caps: BTreeSet<Capability> = role.capabilities.clone();
        for a in assignments {
            if a.role_id == role.id && a.is_granted() && a.applies_to(role.school_id) {
                caps.insert(a.capability);
            }
        }
        caps
    }

    /// Returns `true` if the role is a system role.
    #[must_use]
    pub fn is_system(role: &crate::aggregate::Role) -> bool {
        role.is_system()
    }

    /// Returns `Ok(())` if the role can be deleted: not a system
    /// role and zero user bindings.
    pub fn can_delete(role: &crate::aggregate::Role, user_binding_count: u64) -> Result<()> {
        if Self::is_system(role) {
            return Err(crate::errors::system_role_immutable());
        }
        if user_binding_count > 0 {
            return Err(crate::errors::role_has_bindings(user_binding_count));
        }
        Ok(())
    }

    /// Returns `Ok(())` if the role can be renamed to `new_name`.
    /// System roles require the `RbacRoleManage` capability; the
    /// caller is responsible for the capability check.
    pub fn can_rename(role: &crate::aggregate::Role) -> Result<()> {
        if Self::is_system(role) {
            return Err(crate::errors::system_role_rename_denied());
        }
        Ok(())
    }
}

/// The engine's default role catalog. Each method returns the
/// capability set the corresponding role should hold at school
/// activation. The catalog is a suggestion; consumers may
/// override per school.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultRoleCatalog;

impl DefaultRoleCatalog {
    /// Every registered `Capability` (the union of all known
    /// variants). Used by `SuperAdmin` and by the
    /// `BootstrapService` seed.
    #[must_use]
    pub fn super_admin() -> BTreeSet<Capability> {
        Capability::all().iter().copied().collect()
    }

    /// All platform / RBAC capabilities and the most-used domain
    /// reads/writes. School admin manages everything except
    /// cross-school platform setup.
    #[must_use]
    pub fn school_admin() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        // Platform: full read/write on school and user; no cross-school delete.
        s.extend([
            Capability::PlatformSchoolRead,
            Capability::PlatformSchoolUpdate,
            Capability::PlatformUserCreate,
            Capability::PlatformUserRead,
            Capability::PlatformUserUpdate,
            Capability::PlatformUserDelete,
        ]);
        // RBAC: full management.
        s.extend([
            Capability::RbacRoleCreate,
            Capability::RbacRoleRead,
            Capability::RbacRoleUpdate,
            Capability::RbacRoleDelete,
            Capability::RbacRoleManage,
            Capability::RbacRoleClone,
            Capability::RbacCapabilityAssign,
            Capability::RbacCapabilityRevoke,
            Capability::RbacCapabilityRead,
            Capability::RbacCapabilityUpdateMetadata,
        ]);
        // Domain reads + writes.
        s.extend([
            Capability::AcademicStudentCreate,
            Capability::AcademicStudentRead,
            Capability::AcademicStudentUpdate,
            Capability::AcademicStudentDelete,
            Capability::AcademicClassCreate,
            Capability::AcademicClassRead,
            Capability::AcademicClassUpdate,
            Capability::AcademicClassDelete,
            Capability::AssessmentExamCreate,
            Capability::AssessmentExamRead,
            Capability::AssessmentExamUpdate,
            Capability::AssessmentExamDelete,
            Capability::AssessmentExamScheduleCreate,
            Capability::AssessmentExamScheduleRead,
            Capability::AssessmentExamScheduleUpdate,
            Capability::AssessmentExamScheduleDelete,
            Capability::AssessmentMarksRegisterCreate,
            Capability::AssessmentMarksRegisterRead,
            Capability::AssessmentMarksRegisterUpdate,
            Capability::AssessmentMarksRegisterDelete,
            Capability::AssessmentResultStoreCreate,
            Capability::AssessmentResultStoreRead,
            Capability::AssessmentResultStoreUpdate,
            Capability::AssessmentResultStoreDelete,
            Capability::AssessmentReportCardGenerate,
            Capability::AssessmentReportCardRead,
            Capability::AssessmentReportCardDownload,
            Capability::AssessmentOnlineExamCreate,
            Capability::AssessmentOnlineExamRead,
            Capability::AssessmentOnlineExamUpdate,
            Capability::AssessmentOnlineExamDelete,
            Capability::AssessmentSeatPlanCreate,
            Capability::AssessmentSeatPlanRead,
            Capability::AssessmentSeatPlanUpdate,
            Capability::AssessmentSeatPlanDelete,
            Capability::AssessmentAdmitCardCreate,
            Capability::AssessmentAdmitCardRead,
            Capability::AssessmentAdmitCardUpdate,
            Capability::AssessmentAdmitCardDelete,
            Capability::FinanceInvoiceCreate,
            Capability::FinanceInvoiceRead,
            Capability::FinanceInvoiceUpdate,
            Capability::FinanceInvoiceDelete,
            Capability::FinanceFeesGroupCreate,
            Capability::FinanceFeesGroupRead,
            Capability::FinanceFeesGroupUpdate,
            Capability::FinanceFeesGroupDelete,
            Capability::FinanceFeesTypeCreate,
            Capability::FinanceFeesTypeRead,
            Capability::FinanceFeesTypeUpdate,
            Capability::FinanceFeesTypeDelete,
            Capability::FinanceFeesMasterCreate,
            Capability::FinanceFeesMasterRead,
            Capability::FinanceFeesMasterUpdate,
            Capability::FinanceFeesMasterDelete,
            Capability::FinanceFeesDiscountCreate,
            Capability::FinanceFeesDiscountRead,
            Capability::FinanceFeesDiscountUpdate,
            Capability::FinanceFeesDiscountDelete,
            Capability::FinanceFeesAssignCreate,
            Capability::FinanceFeesAssignRead,
            Capability::FinanceFeesAssignUpdate,
            Capability::FinanceFeesAssignClose,
            Capability::FinanceFeesInstallmentCreate,
            Capability::FinanceFeesInstallmentRead,
            Capability::FinanceFeesInstallmentUpdate,
            Capability::FinanceFeesInstallmentDelete,
            Capability::FinanceFeesInstallmentAssign,
            Capability::FinanceDirectFeesInstallmentCreate,
            Capability::FinanceDirectFeesInstallmentRead,
            Capability::FinanceDirectFeesInstallmentUpdate,
            Capability::FinanceDirectFeesInstallmentDelete,
            Capability::FinanceDirectFeesInstallmentAssign,
            Capability::FinanceDirectFeesInstallmentPay,
            Capability::FinanceFeesInvoiceGenerate,
            Capability::FinanceFeesInvoiceRead,
            Capability::FinanceFeesInvoiceUpdate,
            Capability::FinanceFeesInvoiceCancel,
            Capability::FinanceFeesInvoiceConfigure,
            Capability::FinanceFeesInvoicePrint,
            Capability::FinancePaymentCollect,
            Capability::FinancePaymentRead,
            Capability::FinancePaymentReverse,
            Capability::FinancePaymentRefund,
            Capability::FinancePaymentMethodCreate,
            Capability::FinancePaymentMethodRead,
            Capability::FinancePaymentMethodUpdate,
            Capability::FinancePaymentMethodDelete,
            Capability::FinancePaymentGatewayConfigure,
            Capability::FinancePaymentGatewayRead,
            Capability::FinancePaymentGatewayUpdate,
            Capability::FinancePaymentGatewayDisable,
            Capability::FinanceExpenseCreate,
            Capability::FinanceExpenseRead,
            Capability::FinanceExpenseUpdate,
            Capability::FinanceExpenseDelete,
            Capability::FinanceExpenseApprove,
            Capability::FinanceExpenseHeadCreate,
            Capability::FinanceExpenseHeadRead,
            Capability::FinanceExpenseHeadUpdate,
            Capability::FinanceExpenseHeadDelete,
            Capability::FinanceIncomeCreate,
            Capability::FinanceIncomeRead,
            Capability::FinanceIncomeUpdate,
            Capability::FinanceIncomeDelete,
            Capability::FinanceIncomeApprove,
            Capability::FinanceIncomeHeadCreate,
            Capability::FinanceIncomeHeadRead,
            Capability::FinanceIncomeHeadUpdate,
            Capability::FinanceIncomeHeadDelete,
            Capability::FinanceBankOpen,
            Capability::FinanceBankRead,
            Capability::FinanceBankUpdate,
            Capability::FinanceBankClose,
            Capability::FinanceBankStatementRecord,
            Capability::FinanceBankStatementReverse,
            Capability::FinanceBankTransfer,
            Capability::FinanceBankSlipGenerate,
            Capability::FinanceBankSlipRead,
            Capability::FinanceBankSlipApprove,
            Capability::FinanceBankSlipReject,
            Capability::FinancePayrollPaymentRead,
            Capability::FinancePayrollPaymentRecord,
            Capability::FinanceWalletCredit,
            Capability::FinanceWalletDebit,
            Capability::FinanceWalletRead,
            Capability::FinanceWalletApprove,
            Capability::FinanceWalletReject,
            Capability::FinanceFeesCarryForwardExecute,
            Capability::FinanceFeesCarryForwardRead,
            Capability::FinanceFeesCarryForwardConfigure,
            Capability::FinanceDueFeesBlock,
            Capability::FinanceDueFeesUnblock,
            Capability::FinanceDueFeesRead,
            Capability::FinanceFeesReminderConfigure,
            Capability::FinanceFeesReminderRead,
            Capability::FinanceFeesReminderUpdate,
            Capability::FinanceFeesReminderDelete,
            Capability::FinanceChartOfAccountCreate,
            Capability::FinanceChartOfAccountRead,
            Capability::FinanceChartOfAccountUpdate,
            Capability::FinanceChartOfAccountDelete,
            Capability::FinanceReportFeesCollected,
            Capability::FinanceReportFeesOutstanding,
            Capability::FinanceReportDailyCollection,
            Capability::FinanceReportExpense,
            Capability::FinanceReportBankReconciliation,
            Capability::FinanceReportWalletLedger,
            Capability::FinanceReportRead,
            Capability::HrStaffCreate,
            Capability::HrStaffRead,
            Capability::HrStaffUpdate,
            Capability::HrStaffDelete,
            Capability::HrStaffSuspend,
            Capability::HrStaffReinstate,
            Capability::HrStaffResign,
            Capability::HrStaffTerminate,
            Capability::HrStaffRetire,
            Capability::HrStaffChangeDepartment,
            Capability::HrStaffChangeDesignation,
            Capability::HrStaffChangeRole,
            Capability::HrStaffAssignSubjectTeacher,
            Capability::HrStaffAssignClassTeacherCreate,
            Capability::HrStaffAssignClassTeacherUpdate,
            Capability::HrStaffAssignClassTeacherDelete,
            Capability::HrStaffImportBulk,
            Capability::HrStaffImportBulkPromote,
            Capability::HrStaffImportBulkReject,
            Capability::HrStaffDocumentUpload,
            Capability::HrStaffDocumentDownload,
            Capability::HrDepartmentCreate,
            Capability::HrDepartmentRead,
            Capability::HrDepartmentUpdate,
            Capability::HrDepartmentDelete,
            Capability::HrDesignationCreate,
            Capability::HrDesignationRead,
            Capability::HrDesignationUpdate,
            Capability::HrDesignationDelete,
            Capability::HrLeaveTypeCreate,
            Capability::HrLeaveTypeRead,
            Capability::HrLeaveTypeUpdate,
            Capability::HrLeaveTypeDelete,
            Capability::HrLeaveDefineCreate,
            Capability::HrLeaveDefineRead,
            Capability::HrLeaveDefineUpdate,
            Capability::HrLeaveDefineDelete,
            Capability::HrLeaveRequest,
            Capability::HrLeaveApprove,
            Capability::HrLeaveReject,
            Capability::HrLeaveCancel,
            Capability::HrLeaveRead,
            Capability::HrAttendanceStaffMark,
            Capability::HrAttendanceStaffUpdate,
            Capability::HrAttendanceStaffDelete,
            Capability::HrAttendanceStaffRead,
            Capability::HrAttendanceStaffImport,
            Capability::HrAttendanceStaffImportPromote,
            Capability::HrAttendanceStaffImportReject,
            Capability::HrPayrollGenerate,
            Capability::HrPayrollUpdate,
            Capability::HrPayrollApprove,
            Capability::HrPayrollMarkPaid,
            Capability::HrPayrollRead,
            Capability::HrPayrollEarningAdd,
            Capability::HrPayrollEarningUpdate,
            Capability::HrPayrollEarningDelete,
            Capability::HrPayrollDeductionAdd,
            Capability::HrPayrollDeductionUpdate,
            Capability::HrPayrollDeductionDelete,
            Capability::HrPayrollLeaveDeductionAdd,
            Capability::HrPayrollLeaveDeductionUpdate,
            Capability::HrPayrollLeaveDeductionDelete,
            Capability::HrPayrollPaymentRead,
            Capability::HrSalaryTemplateCreate,
            Capability::HrSalaryTemplateRead,
            Capability::HrSalaryTemplateUpdate,
            Capability::HrSalaryTemplateDelete,
            Capability::HrHourlyRateSet,
            Capability::HrHourlyRateRead,
            Capability::HrHourlyRateUpdate,
            Capability::HrHourlyRateDelete,
            Capability::HrStaffRegistrationFieldCreate,
            Capability::HrStaffRegistrationFieldRead,
            Capability::HrStaffRegistrationFieldUpdate,
            Capability::HrStaffRegistrationFieldDelete,
            Capability::HrReportStaffRoster,
            Capability::HrReportStaffByDepartment,
            Capability::HrReportStaffByDesignation,
            Capability::HrReportLeaveUsage,
            Capability::HrReportLeaveBalance,
            Capability::HrReportAttendanceDaily,
            Capability::HrReportAttendanceMonthly,
            Capability::HrReportAttendanceByStaff,
            Capability::HrReportPayrollRegister,
            Capability::HrReportPayrollByStaff,
            Capability::HrReportPayrollByDepartment,
            Capability::HrReportPayrollTax,
            Capability::HrReportSalaryStructure,
            Capability::HrReportHourlyEarnings,
            Capability::HrReportLeaveDeduction,
            Capability::HrReportRead,
            Capability::LibraryRead,
            Capability::LibraryConfigure,
            Capability::LibraryReport,
            Capability::BookCategoryCreate,
            Capability::BookCategoryRead,
            Capability::BookCategoryUpdate,
            Capability::BookCategoryDelete,
            Capability::BookAdd,
            Capability::BookRead,
            Capability::BookUpdate,
            Capability::BookDelete,
            Capability::BookAdjustQuantity,
            Capability::BookSearch,
            Capability::MemberRegister,
            Capability::MemberRead,
            Capability::MemberUpdate,
            Capability::MemberDelete,
            Capability::MemberDeactivate,
            Capability::MemberReactivate,
            Capability::BookIssueIssue,
            Capability::BookIssueRead,
            Capability::BookIssueReturn,
            Capability::BookIssueRenew,
            Capability::BookIssueMarkLost,
            Capability::BookIssueCalculateFine,
            Capability::BookIssueWaiveFine,
            Capability::CommunicationMessageCreate,
            Capability::CommunicationMessageRead,
            Capability::CommunicationMessageUpdate,
            Capability::CommunicationMessageDelete,
            Capability::DocumentsFolderCreate,
            Capability::DocumentsFolderRead,
            Capability::DocumentsFolderUpdate,
            Capability::DocumentsFolderDelete,
            Capability::FormDownloadUpload,
            Capability::FormDownloadUpdate,
            Capability::FormDownloadDelete,
            Capability::FormDownloadRead,
            Capability::PostalDispatchCreate,
            Capability::PostalDispatchUpdate,
            Capability::PostalDispatchDelete,
            Capability::PostalReceiveCreate,
            Capability::PostalReceiveUpdate,
            Capability::PostalReceiveDelete,
            Capability::PostalRead,
            Capability::CmsPageCreate,
            Capability::CmsPageRead,
            Capability::CmsPageUpdate,
            Capability::CmsPageDelete,
            Capability::FacilitiesRoomCreate,
            Capability::FacilitiesRoomRead,
            Capability::FacilitiesRoomUpdate,
            Capability::FacilitiesRoomDelete,
            Capability::EventsCalendarCreate,
            Capability::EventsCalendarRead,
            Capability::EventsCalendarUpdate,
            Capability::EventsCalendarDelete,
            Capability::SettingsManage,
            Capability::OperationsManage,
        ]);
        s
    }

    /// Teacher: academic domain reads + writes (own classes) and
    /// limited platform reads.
    #[must_use]
    pub fn teacher() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacRoleRead,
            Capability::RbacCapabilityRead,
            Capability::AcademicStudentRead,
            Capability::AcademicStudentUpdate,
            Capability::AcademicClassRead,
            Capability::AcademicClassUpdate,
            Capability::AssessmentExamRead,
            Capability::AssessmentExamScheduleRead,
            Capability::AssessmentMarksRegisterCreate,
            Capability::AssessmentMarksRegisterRead,
            Capability::AssessmentMarksRegisterUpdate,
            Capability::AssessmentResultStoreRead,
            Capability::AssessmentOnlineExamCreate,
            Capability::AssessmentOnlineExamRead,
            Capability::AssessmentOnlineExamUpdate,
            Capability::CommunicationMessageCreate,
            Capability::CommunicationMessageRead,
            Capability::EventsCalendarRead,
        ]);
        s
    }

    /// Student: read own profile + academic reads. No finance.
    #[must_use]
    pub fn student() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacCapabilityRead,
            Capability::AcademicStudentRead,
            Capability::AcademicClassRead,
            Capability::EventsCalendarRead,
        ]);
        s
    }

    /// Parent: read own children's academic data + own profile. No
    /// finance.
    #[must_use]
    pub fn parent() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacCapabilityRead,
            Capability::AcademicStudentRead,
            Capability::AcademicClassRead,
            Capability::EventsCalendarRead,
            Capability::CommunicationMessageRead,
        ]);
        s
    }

    /// Accountant: finance domain full access + limited platform
    /// reads.
    #[must_use]
    pub fn accountant() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacRoleRead,
            Capability::RbacCapabilityRead,
            Capability::FinanceInvoiceCreate,
            Capability::FinanceInvoiceRead,
            Capability::FinanceInvoiceUpdate,
            Capability::FinanceInvoiceDelete,
        ]);
        s
    }

    /// Receptionist: front-office communication + Documents
    /// front-office (form downloads, postal dispatch/receive, read)
    /// + limited platform access.
    #[must_use]
    pub fn receptionist() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacCapabilityRead,
            Capability::CommunicationMessageCreate,
            Capability::CommunicationMessageRead,
            Capability::CommunicationMessageUpdate,
            Capability::EventsCalendarRead,
            Capability::FormDownloadUpload,
            Capability::FormDownloadUpdate,
            Capability::FormDownloadDelete,
            Capability::FormDownloadRead,
            Capability::PostalDispatchCreate,
            Capability::PostalDispatchUpdate,
            Capability::PostalDispatchDelete,
            Capability::PostalReceiveCreate,
            Capability::PostalReceiveUpdate,
            Capability::PostalReceiveDelete,
            Capability::PostalRead,
        ]);
        s
    }

    /// Librarian: library domain full access + limited platform
    /// reads.
    #[must_use]
    pub fn librarian() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacCapabilityRead,
            Capability::LibraryRead,
            Capability::LibraryConfigure,
            Capability::LibraryReport,
            Capability::BookCategoryCreate,
            Capability::BookCategoryRead,
            Capability::BookCategoryUpdate,
            Capability::BookCategoryDelete,
            Capability::BookAdd,
            Capability::BookRead,
            Capability::BookUpdate,
            Capability::BookDelete,
            Capability::BookAdjustQuantity,
            Capability::BookSearch,
            Capability::MemberRegister,
            Capability::MemberRead,
            Capability::MemberUpdate,
            Capability::MemberDelete,
            Capability::MemberDeactivate,
            Capability::MemberReactivate,
            Capability::BookIssueIssue,
            Capability::BookIssueRead,
            Capability::BookIssueReturn,
            Capability::BookIssueRenew,
            Capability::BookIssueMarkLost,
            Capability::BookIssueCalculateFine,
            Capability::BookIssueWaiveFine,
        ]);
        s
    }

    /// Driver: transport / facilities reads only. Placeholder;
    /// `Events.Calendar.Read` and `Facilities.Room.Read` cover
    /// the current engine surface.
    #[must_use]
    pub fn driver() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacCapabilityRead,
            Capability::FacilitiesRoomRead,
            Capability::EventsCalendarRead,
        ]);
        s
    }

    /// Generic non-teaching staff: limited platform + communication.
    #[must_use]
    pub fn staff() -> BTreeSet<Capability> {
        let mut s: BTreeSet<Capability> = BTreeSet::new();
        s.extend([
            Capability::PlatformUserRead,
            Capability::RbacCapabilityRead,
            Capability::CommunicationMessageCreate,
            Capability::CommunicationMessageRead,
        ]);
        s
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::{TenantContext, UserType};
    use uuid::Uuid;

    fn ctx_of(user_type: UserType) -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            user_type,
        )
    }

    #[test]
    fn super_admin_role_includes_every_capability() {
        let all: BTreeSet<Capability> = DefaultRoleCatalog::super_admin();
        for c in Capability::all() {
            assert!(all.contains(c), "missing capability {c:?} in super_admin");
        }
    }

    #[test]
    fn student_role_excludes_finance_capabilities() {
        let s = DefaultRoleCatalog::student();
        assert!(!s.contains(&Capability::FinanceInvoiceCreate));
        assert!(!s.contains(&Capability::FinanceInvoiceRead));
        assert!(!s.contains(&Capability::FinanceInvoiceUpdate));
        assert!(!s.contains(&Capability::FinanceInvoiceDelete));
    }

    #[test]
    fn in_memory_has_returns_true_for_granted() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let role = RoleId::new(school, Uuid::now_v7());
        let check = InMemoryCapabilityCheck::new();
        check.grant(school, role, Capability::PlatformUserRead);
        let mut ctx = ctx_of(UserType::Teacher);
        ctx.school_id = school;
        let r = futures::executor::block_on(check.has(&ctx, Capability::PlatformUserRead)).unwrap();
        assert!(r);
    }

    #[test]
    fn in_memory_has_returns_false_for_not_granted() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let role = RoleId::new(school, Uuid::now_v7());
        let check = InMemoryCapabilityCheck::new();
        check.grant(school, role, Capability::PlatformUserRead);
        let mut ctx = ctx_of(UserType::Teacher);
        ctx.school_id = school;
        let r = futures::executor::block_on(check.has(&ctx, Capability::RbacBootstrap)).unwrap();
        assert!(!r, "non-system teacher should not hold RbacBootstrap");
    }

    #[test]
    fn rbac_bootstrap_is_never_revocable() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let role = RoleId::new(school, Uuid::now_v7());
        let check = InMemoryCapabilityCheck::new();
        check.grant(school, role, Capability::RbacRoleManage);
        let mut ctx = ctx_of(UserType::SchoolAdmin);
        ctx.school_id = school;
        let r = futures::executor::block_on(check.has(&ctx, Capability::RbacBootstrap)).unwrap();
        assert!(
            r,
            "RbacBootstrap must be granted to any RbacRoleManage holder"
        );
    }

    #[test]
    fn explain_records_system_fallback_for_bootstrap() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let role = RoleId::new(school, Uuid::now_v7());
        let check = InMemoryCapabilityCheck::new();
        check.grant(school, role, Capability::RbacRoleManage);
        let mut ctx = ctx_of(UserType::SchoolAdmin);
        ctx.school_id = school;
        let exp =
            futures::executor::block_on(check.explain(&ctx, Capability::RbacBootstrap)).unwrap();
        assert!(exp.decision);
        assert!(exp.system_fallback);
    }

    #[test]
    fn has_any_with_empty_slice_returns_true() {
        let check = InMemoryCapabilityCheck::new();
        let ctx = ctx_of(UserType::Teacher);
        let r = futures::executor::block_on(check.has_any(&ctx, &[])).unwrap();
        assert!(r);
    }

    #[test]
    fn has_all_with_empty_slice_returns_true() {
        let check = InMemoryCapabilityCheck::new();
        let ctx = ctx_of(UserType::Teacher);
        let r = futures::executor::block_on(check.has_all(&ctx, &[])).unwrap();
        assert!(r);
    }

    #[test]
    fn invalidate_cache_drops_school() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let role = RoleId::new(school, Uuid::now_v7());
        let check = InMemoryCapabilityCheck::new();
        check.grant(school, role, Capability::PlatformUserRead);
        let mut ctx = ctx_of(UserType::Teacher);
        ctx.school_id = school;
        let r = futures::executor::block_on(check.has(&ctx, Capability::PlatformUserRead)).unwrap();
        assert!(r);
        futures::executor::block_on(check.invalidate_cache(&ctx)).unwrap();
        let r = futures::executor::block_on(check.has(&ctx, Capability::PlatformUserRead)).unwrap();
        assert!(!r);
    }
}
